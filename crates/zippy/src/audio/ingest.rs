use core::time::Duration;

use anyhow::{Result, anyhow};
use backon::{ExponentialBuilder, Retryable, RetryableWithContext};
use es7210::{CodecConfig, I2sBits, I2sFormat, MicBias, MicGain};
use esp_hal::{
    i2c::master::{Config, I2c},
    i2s::master::{Channels, DataFormat, I2s, TdmConfig},
    peripherals::{DMA_CH0, GPIO10, GPIO11, I2C0, I2S0},
    time::Rate,
};

use crate::{configuration::I2C_ADDRESS, eventbus::Events};

struct Args<'a> {
    i2s: &'a mut I2S0<'static>,
    i2c: &'a mut I2C0<'static>,
    dma: &'a mut DMA_CH0<'static>,
    gpio10: &'a mut GPIO10<'static>,
    gpio11: &'a mut GPIO11<'static>,
    attempt: usize,
}

#[embassy_executor::task]
pub async fn audio(
    mut i2s: I2S0<'static>,
    mut i2c: I2C0<'static>,
    mut gpio10: GPIO10<'static>,
    mut gpio11: GPIO11<'static>,
    mut dma: DMA_CH0<'static>,
) {
    tracing::info!("starting TCP/UDP channels");
    let mut args = Args {
        i2s: &mut i2s,
        i2c: &mut i2c,
        dma: &mut dma,
        gpio10: &mut gpio10,
        gpio11: &mut gpio11,
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
    .notify(|err: &anyhow::Error, dur: Duration| {
        tracing::info!("encountered `{err}` retrying after {dur:?}");
    })
    .await;

    if let Err(e) = task {
        panic!("unrecoverable error occured in net task {e}");
    }
}

async fn run(args: &mut Args<'_>) -> Result<()> {
    let Args { i2s, dma, i2c, gpio10, gpio11, .. } = args;

    // setup the ES7210 chip (for microphone) and i2c bus (at address 0x40, 0x41, 0x43)
    let config = TdmConfig::new_tdm_philips()
        .with_sample_rate(Rate::from_hz(44100))
        .with_data_format(DataFormat::Data16Channel16)
        .with_channels(Channels::STEREO);
    let i2s = I2s::new(i2s.reborrow(), dma.reborrow(), config)?;
    let mut i2c = I2c::new(i2c.reborrow(), Config::default().with_frequency(Rate::from_khz(100)))?
        .with_scl(gpio10.reborrow())
        .with_sda(gpio11.reborrow());
    let codec = es7210::Es7210::new(I2C_ADDRESS);
    let cfg = CodecConfig {
        sample_rate_hz: 48_000,
        mclk_ratio: 256,
        i2s_format: I2sFormat::I2S,
        bit_width: I2sBits::Bits24,
        mic_bias: MicBias::V2_55,
        mic_gain: MicGain::Gain18dB,
        tdm_enable: false,
    };
    codec.config_codec(&mut i2c, &cfg).map_err(|_| anyhow!("codec configuration failed"))?;
    let mut subscriber = crate::eventbus::subscriber()?;
    loop {
        match subscriber.next_message_pure().await {
            Events::BeginListening => {
                todo!()
            }
            _ => continue,
        }
        args.attempt = 0;
    }
    Ok(())
}
