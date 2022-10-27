import { Action, CardId, TargetId } from "./rustTypes";

export type CardActions = {
    [key: CardId]: {
        action: Action,
        index: number
    }[]
};

export type ActionsUnion = {
    type: "none"
} | {
    type: "action",
    actions: CardActions
} | {
    type: "attackers",
    attackers: { [k: CardId]: [number, number] },
    response: { [k: CardId]: (TargetId | null) }
    targets: TargetId[],
    selected_attacker: CardId | null,
} | {
    type: "blockers",
    blockers: { [k: CardId]: [number, number] },
    attackers: CardId[],
    response: { [k: CardId]: [TargetId] },
    selected_blocker: CardId | null,
};