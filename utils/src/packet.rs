enum FlagState {
    WARNING,
    COLLISION,
    COORDINATE,
    EXIT,
}

struct Packet {
    flag: FlagState
}
