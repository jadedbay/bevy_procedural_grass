pub struct Clump {
    direction: ClumpDirection,
}

pub enum ClumpDirection {
    In,
    Out,
    Same,
    Random,
}

