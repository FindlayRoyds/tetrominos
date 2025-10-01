use rand_pcg::Pcg32;

#[derive(bevy::prelude::Resource)]
pub struct RandomSource(pub Pcg32);
