
use libspa::utils::Direction;
use libspa_sys::{spa_audio_channel, spa_direction};

pub fn channel_name_to_spa_audio_channel(name: &str) -> spa_audio_channel {
    // TODO there should be a mapping like this in PW or libSPA???
    // TODO the rest of these
    let channel = match name {
        "FL" => libspa_sys::SPA_AUDIO_CHANNEL_FL,
        "FR" => libspa_sys::SPA_AUDIO_CHANNEL_FR,
        _ => libspa_sys::SPA_AUDIO_CHANNEL_UNKNOWN,
    };
    return channel;
}

// use .as_raw() for spa_direction
pub fn direction_name_to_spa_direction(name: &str) -> libspa::utils::Direction {
    let direction = match name {
        "in" => libspa::utils::Direction::Input,
        "out" => libspa::utils::Direction::Output,
        _ => panic!("spa_direction only has in and out"),
    };
    return direction;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_channel_mappings() {
        assert_eq!(
            channel_name_to_spa_audio_channel("FR"),
            libspa_sys::SPA_AUDIO_CHANNEL_FR
        );
        assert_eq!(
            channel_name_to_spa_audio_channel("FL"),
            libspa_sys::SPA_AUDIO_CHANNEL_FL
        );
    }
}
