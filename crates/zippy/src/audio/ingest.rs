use anyhow::{Result, anyhow};
use backon::{ExponentialBuilder, Retryable, RetryableWithContext};
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    zerocopy_channel::{Receiver, Sender},
};
use embassy_time::{Duration, Instant};
use es7210::{CodecConfig, I2sBits, I2sFormat, MicBias, MicGain};
use esp_hal::{
    Async,
    i2c::master::{Config, I2c},
    i2s::master::{Channels, DataFormat, I2s, I2sRx, TdmConfig},
    peripherals::{DMA_CH0, GPIO10, GPIO11, GPIO12, GPIO13, GPIO14, GPIO15, I2C0, I2S0},
    time::Rate,
};

use crate::{configuration::I2C_ADDRESS, eventbus::Events};

#[derive(Copy, Clone)]
pub struct AudioChunk {
    /// Order of this chunk in the audio stream
    pub sequence: u16,
    /// chunk of 16-bit PCM, 10ms of audio at 24 kHz mono
    pub pcm: [u8; 960],
}

impl AudioChunk {
    pub fn new(sequence: u16) -> Self {
        Self { sequence, pcm: [0u8; 960] }
    }
}

pub type IngestReceiver<'a> = Receiver<'a, NoopRawMutex, AudioChunk>;

pub type IngestSender<'a> = Sender<'a, NoopRawMutex, AudioChunk>;

#[derive(bon::Builder)]
pub struct MicrophoneGpio {
    gpio10: GPIO10<'static>,
    gpio11: GPIO11<'static>,
    gpio12: GPIO12<'static>,
    gpio13: GPIO13<'static>,
    gpio14: GPIO14<'static>,
    gpio15: GPIO15<'static>,
}

struct Args<'a> {
    i2s: &'a mut I2S0<'static>,
    i2c: &'a mut I2C0<'static>,
    dma: &'a mut DMA_CH0<'static>,
    gpio: &'a mut MicrophoneGpio,
    /// channel to the network task
    tx: &'a mut IngestSender<'static>,
    attempt: usize,
}

#[embassy_executor::task]
pub async fn ingest(
    mut i2s: I2S0<'static>,
    mut i2c: I2C0<'static>,
    mut gpio: MicrophoneGpio,
    mut dma: DMA_CH0<'static>,
    mut tx: IngestSender<'static>,
) {
    tracing::info!("starting Audio ingest");
    let mut args = Args {
        i2s: &mut i2s,
        i2c: &mut i2c,
        dma: &mut dma,
        gpio: &mut gpio,
        tx: &mut tx,
        attempt: 0,
    };

    let (_, task) = {
        async |mut args| {
            let result = run(&mut args).await;
            args.attempt += 1;
            (args, result)
        }
    }
    .retry(ExponentialBuilder::default().without_max_times())
    .sleep(backon::EmbassySleeper)
    .context(args)
    .notify(|err: &anyhow::Error, dur: core::time::Duration| {
        tracing::info!("encountered `{err}` retrying after {dur:?}");
    })
    .await;

    if let Err(e) = task {
        panic!("unrecoverable error occured in net task {e}");
    }
}

async fn run(args: &mut Args<'_>) -> Result<()> {
    let Args { i2s, dma, i2c, gpio, tx, .. } = args;

    // setup the ES7210 chip (for microphone) and i2c bus (at address 0x40, 0x41, 0x43)
    let config = TdmConfig::new_tdm_philips()
        .with_sample_rate(Rate::from_hz(48_000))
        .with_data_format(DataFormat::Data16Channel16)
        .with_channels(Channels::LEFT);
    let mut i2c = I2c::new(i2c.reborrow(), Config::default().with_frequency(Rate::from_khz(100)))?
        .with_scl(gpio.gpio10.reborrow())
        .with_sda(gpio.gpio11.reborrow());
    let codec = es7210::Es7210::new(I2C_ADDRESS);
    let cfg = CodecConfig {
        sample_rate_hz: 48_000,
        mclk_ratio: 256,
        i2s_format: I2sFormat::I2S,
        bit_width: I2sBits::Bits16,
        mic_bias: MicBias::V2_55,
        mic_gain: MicGain::Gain18dB,
        tdm_enable: false,
    };
    codec.config_codec(&mut i2c, &cfg).map_err(|_| anyhow!("codec configuration failed"))?;
    let mut subscriber = crate::eventbus::subscriber()?;
    loop {
        match subscriber.next_message_pure().await {
            Events::BeginListening => {
                capture_audio_for(Duration::from_secs(10), i2s, dma, config, gpio, tx).await?
            }
            _ => continue,
        }
        args.attempt = 0;
    }
    Ok(())
}

/// Capture 'Duration' of audio
async fn capture_audio_for(
    capture_duration: Duration,
    i2s: &mut I2S0<'static>,
    dma: &mut DMA_CH0<'static>,
    config: TdmConfig,
    gpio: &mut MicrophoneGpio,
    tx: &mut IngestSender<'_>,
) -> Result<()> {
    tracing::info!(capture_duration_ms = capture_duration.as_millis(), "listening for audio");
    // | Size                            | Capture time           |
    // |---------------------------------|------------------------|
    // | 960 bytes per DMA block         | 5 ms                   |
    // | 1,920 bytes in the audio array  | 10 ms                  |
    // | 7,680 bytes in the DMA ring     | 40 ms                  |
    let mut i2s = I2s::new(i2s.reborrow(), dma.reborrow(), config)?.into_async();
    let rx = i2s
        .with_mclk(gpio.gpio12.reborrow())
        .i2s_rx
        .with_bclk(gpio.gpio13.reborrow())
        .with_ws(gpio.gpio14.reborrow())
        .with_din(gpio.gpio15.reborrow())
        .build();
    let buf = esp_hal::dma_rx_stream_buffer!(7680, 960);
    let mut capture = rx.read(buf).map_err(|e| anyhow!("failed to capture audio {e:?}"))?;
    let mut audio = [0u8; 960];
    let mut chunks_since_log = 0;
    let mut now = Instant::now();
    let until = now + capture_duration;
    let mut sequence = 0;
    while now <= until {
        let mut filled = 0;
        while filled < audio.len() {
            if capture.available_bytes() == 0 {
                capture
                    .wait_for_available_async()
                    .await
                    .map_err(|e| anyhow!("failed to wait for available bytes from ES7210 {e:?}"))?;
            }
            // I2sRxDmaTransfer derefs to DmaRxStreamBufView as DmaRxStreamBufView is the
            // DmaRxBuffer implementation for 'Buf' generic on I2sRxDmaTransfer.
            // https://docs.espressif.com/projects/rust/esp-hal/1.2.0/esp32s3/esp_hal/dma/struct.DmaRxStreamBufView.html
            filled += capture.pop(&mut audio[filled..]);
        }
        tracing::info!("awaiting send slot");
        let mut buf = tx.send().await;
        *buf = AudioChunk { sequence, pcm: audio };
        tx.send_done();
        tracing::info!("sent");
        now = Instant::now();
    }
    Ok(())
}
