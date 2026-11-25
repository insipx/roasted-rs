use core::time::Duration;

use anyhow::Result;
use backon::{ExponentialBuilder, Retryable, RetryableWithContext};
use esp_hal::{
    i2s::master::{Channels, DataFormat, I2s, TdmConfig},
    peripherals::{DMA_CH0, I2S0},
    time::Rate,
};

struct Args<'a> {
    i2s: &'a mut I2S0<'static>,
    dma: &'a mut DMA_CH0<'static>,
    attempt: usize,
}

#[embassy_executor::task]
pub async fn audio(mut i2s: I2S0<'static>, mut dma: DMA_CH0<'static>) {
    tracing::info!("starting TCP/UDP channels");
    let mut args = Args { i2s: &mut i2s, dma: &mut dma, attempt: 0 };

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
    let Args { i2s, dma, .. } = args;
    let config = TdmConfig::new_tdm_philips()
        .with_sample_rate(Rate::from_hz(44100))
        .with_data_format(DataFormat::Data16Channel16)
        .with_channels(Channels::STEREO);
    let i2s = I2s::new(i2s.reborrow(), dma.reborrow(), config)?;

    let mut subscriber = crate::eventbus::subscriber()?;
    loop {
        match subscriber.next_message_pure().await {
            _ => continue,
        }
        args.attempt = 0;
    }
    Ok(())
}
