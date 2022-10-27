import './style.css';
import React from 'react';
import { ArcherContainer } from 'react-archer';
import { phase_image_map, PhaseImages } from './Phases';
import {
    Ask,
    AskSelectNFor_Action,
    ClientMessage,
    GameState,
} from './rustTypes';
import { CardActions, ActionsUnion } from './actions'
import { PlayerBoxes } from './PlayerBox';

function Stack(props: {
    card_width: number
}) {
    return (
        <div className="stack" style={{ width: props.card_width + "px" }}>

        </div>
    )
}

function process_actions(action_data: AskSelectNFor_Action): CardActions {
    const card_actions: CardActions = {};
    action_data.ents.map((action, index) => {
        let id = null;
        if ("PlayLand" in action) {
            id = action.PlayLand;
        }
        if ("Cast" in action) {
            id = action.Cast.source_card;
        }
        if ("ActivateAbility" in action) {
            id = action.ActivateAbility.source;
        }
        if (!id) {
            console.log("action is");
            console.log(action);
            return null;
        }
        const data = { action: action, index: index };
        if (card_actions[id]) {
            card_actions[id].push(data);
        }
        else {
            card_actions[id] = [data];
        }
        return null;
    });
    return card_actions;
}

interface IProps {
}

interface IState {
    card_width: number,
    playerbox_width: number,
    game: GameState | null,
    handlers: any,
    actions: ActionsUnion
}
function objMap(obj: any, func: any) {
    return Object.fromEntries(Object.entries(obj).map(([k, v]) => [k, func(v)]));
}

function exhaustiveCheck(param: never) { }
class Game extends React.Component<IProps, IState>{
    socket: WebSocket;
    keypresshandler: any;
    constructor(props: {}) {
        super(props);
        const socket = new WebSocket("ws://localhost:3030/gamesetup");
        this.socket = socket;
        this.keypresshandler = this.keyPressed.bind(this);
        socket.addEventListener('message', (event: { data: string; }) => {
            let parsed: ClientMessage = JSON.parse(event.data);
            console.log(parsed);
            if ("GameState" in parsed) {
                this.update_state(parsed.GameState);
            }
            else if ("AskUser" in parsed) {
                this.respond_action(parsed.AskUser);
            }
        });
        this.state = {
            card_width: 75,
            playerbox_width: 125,
            game: null,
            handlers: {
                click: this.item_clicked.bind(this)
            },
            actions: {
                type: "none"
            }
        };
    }
    keyPressed(e: { keyCode: number; }) {
        console.log("key pressed: " + e.keyCode);
        if (e.keyCode !== 32) {
            return;
        }
        if (this.state.actions.type === "none") {
            return;
        }
        if (this.state.actions.type === "action") {
            this.socket.send("[]");
            this.clear_actions();
            return;
        }
        if (this.state.actions.type === "attackers") {
            const resp = objMap(this.state.actions.response, (x: number | null) => {
                if (x === null) {
                    return []
                } else {
                    return [x]
                }
            });
            const response = JSON.stringify(resp);
            this.socket.send(response);
            this.clear_actions();
            return;
        }
        if (this.state.actions.type === "blockers") {
            const resp = JSON.stringify(this.state.actions.response);
            this.socket.send(resp);
            this.clear_actions();
            return;
        }
        exhaustiveCheck(this.state.actions);
    }
    componentDidMount() {
        document.addEventListener("keydown", this.keypresshandler, false);
    }
    componentWillUnmount() {
        document.removeEventListener("keydown", this.keypresshandler, false);
    }
    update_state(parsed: GameState) {
        this.clear_actions();
        this.setState({ game: parsed });
    }
    respond_action(parsed: Ask) {
        let actions: ActionsUnion = {
            type: "none"
        };
        if ("Action" in parsed) {
            const parsed_actions: AskSelectNFor_Action = parsed.Action;
            if (parsed_actions.ents.length === 0) {
                this.socket.send("[]");
                return;
            }
            actions = {
                type: "action",
                actions: process_actions(parsed_actions)
            }
        }
        if ("Attackers" in parsed) {
            const attackers = parsed.Attackers.a;
            if (Object.entries(attackers).length === 0) {
                this.socket.send("{}");
                return;
            }
            actions = {
                type: "attackers",
                attackers: attackers,
                response: objMap(attackers, (_i: any) => null),
                selected_attacker: null,
                targets: parsed.Attackers.b
            }
        }
        if ("Blockers" in parsed) {
            const blockers = parsed.Blockers.a;
            if (Object.entries(blockers).length === 0) {
                this.socket.send("{}");
                return;
            }
            actions = {
                type: "blockers",
                blockers: blockers,
                response: objMap(blockers, (_i: any) => []),
                attackers: parsed.Blockers.b,
                selected_blocker: null,
            }
        }
        this.setState({ actions: actions });
    }
    clear_actions() {
        this.setState({
            actions: {
                type: "none"
            }
        }
        );
    }
    item_clicked(ent_id: number) { //This number is a CardId or PlayerId
        console.log("clicked "+ent_id);
        if (this.state.actions.type === "action") {
            const actions: CardActions = this.state.actions.actions;
            const card_actions = actions[ent_id];
            if (!card_actions) {
                return;
            }
            if (card_actions.length === 1) {
                const to_send = JSON.stringify([card_actions[0].index]);
                this.socket.send(to_send);
                this.clear_actions();
            }
            else {
                throw new Error("I don't know how to deal with multiple actions for one card yet!");
            }
        }
        if (this.state.actions.type === "attackers") {
            const actions = { ...this.state.actions }
            if (ent_id in actions.attackers) {
                if (actions.selected_attacker !== null) {
                    actions.selected_attacker = null;
                } else {
                    actions.selected_attacker = ent_id;
                }
            }
            else if (actions.selected_attacker !== null) {
                const selected = actions.selected_attacker;
                if (actions.targets.includes(ent_id)) {
                    if (actions.response[selected] === ent_id) {
                        actions.response[selected] = null;
                    } else {
                        actions.response[selected] = ent_id;
                    }
                    actions.selected_attacker = null;
                }
            }
            this.setState({ actions: actions });
        }
        if (this.state.actions.type === "blockers") {
            const actions = { ...this.state.actions }
            if (ent_id in actions.blockers) {
                if (actions.selected_blocker !== null) {
                    actions.selected_blocker = null;
                } else {
                    actions.selected_blocker = ent_id;
                }
            }
            else if (actions.selected_blocker !== null) {
                const selected = actions.selected_blocker;
                if (actions.attackers.includes(ent_id)) {
                    const idx = actions.response[selected].indexOf(ent_id);
                    if (idx !== -1) {
                        actions.response[selected].splice(idx, 1);
                    } else {
                        if (actions.response[selected].length < actions.blockers[selected][1]) {
                            actions.response[selected].push(ent_id);
                        }
                    }
                    actions.selected_blocker = null;
                }
            }
            this.setState({ actions: actions });
        }
    }
    render() {
        if (this.state.game) {
            return (
                <ArcherContainer strokeColor="red" svgContainerStyle={{ zIndex: "10" }}>
                    <div className="full-size" style={{ height: "98vh" }}>
                        <PhaseImages phase_image_map={phase_image_map} phase={this.state.game.phase} subphase={this.state.game.subphase} />
                        <Stack card_width={this.state.card_width} />
                        <PlayerBoxes game={this.state.game}
                            width={this.state.playerbox_width}
                            actions={this.state.actions}
                            handlers={this.state.handlers} />
                    </div>
                </ArcherContainer>
            );
        } else {
            return <p> Waiting for game to start</p>
        }
    }
}

export default Game;
