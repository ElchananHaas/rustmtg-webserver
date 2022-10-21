import { Action, CardId,TargetId } from "./rustTypes";

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
    attackers: CardId[],
    targets: TargetId[],
    response: (TargetId | null)[],
    selected_attacker: CardId | null,
};