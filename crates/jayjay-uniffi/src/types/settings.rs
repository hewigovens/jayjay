use jayjay_core::UpdateChannel;

#[uniffi::remote(Enum)]
pub enum UpdateChannel {
    Stable,
    Beta,
}

#[uniffi::export]
fn parse_update_channel(value: String) -> UpdateChannel {
    UpdateChannel::parse(&value)
}

#[uniffi::export]
fn update_channel_identifier(channel: UpdateChannel) -> String {
    channel.identifier().to_owned()
}
