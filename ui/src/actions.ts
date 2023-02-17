import { Action, AskPairItemFor_CardId, AskPairItemFor_TargetId, AskSelectNFor_TargetId, CardId, TargetId } from "./rustTypes";

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
    actions: CardActions,
} | {
    type: "attackers",
    attackers: { [k: CardId]: AskPairItemFor_TargetId };
    response: { [k: CardId]: {
        [k: string]: null;
      } }
    selected_attacker: CardId | null,
} | {
    type: "blockers",
    blockers: { [k: CardId]: AskPairItemFor_CardId },
    response: { [k: CardId]: {
        [k: string]: null;
      } }
    selected_blocker: CardId | null,
} | {
    type: "target",
    action: AskSelectNFor_TargetId,
    response: { [k: CardId]:null},
};