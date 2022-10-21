import './style.css';
import React from 'react';
import { ArcherContainer, ArcherElement } from 'react-archer';
import { phase_image_map, PhaseImages } from './Phases';
import {
    Action,
    Ask,
    AskSelectNFor_Action,
    ClientMessage,
    GameState,
    PlayerId,
    PlayerView,
} from './rustTypes';
import {CardActions,ActionsUnion} from './actions'
import { Card } from './Card';
import { ManaSymbols } from './Mana';
function Stack(props:{
    card_width:number
}) {
    return (
        <div className="stack" style={{ width: props.card_width + "px" }}>

        </div>
    )
}
function ImageCircledText(props:{
    scale?:number,
    rotate?:boolean,
    src:string,
    number:number
}) {
    let rotate:any = {};
    if (props.rotate) {
        rotate = {
            transform: `rotate(90deg)`,
            width: "100%"
        };
    }
    if (props.scale) {
        rotate.height = props.scale + "%";
        rotate.width = props.scale + "%";
    }
    return (<div className="vertical-flexbox">
        <img src={props.src} className="full-height-image" style={rotate} alt=""></img>
        <div className="circle-div">
        </div>
        <p className="text-over-content">
            {props.number}
        </p>
    </div>)
}
function LifeTotalBox(props:{
    actions:ActionsUnion,
    player_id:PlayerId,
    life:number,
    handlers:any
}) {
    let style:{
        borderColor?:string
    } = {};
    const response = props.actions.type === "attackers" && props.actions.response.filter((attackee) => {
        return attackee === props.player_id;
    }).length > 0;
    if (response) {
        style.borderColor = "#AA0000";
    }
    return (
        <ArcherElement id={"playerbox" + props.player_id}>
            <div className="life-total-box" style={style} onClick={() => props.handlers.click(props.player_id)}>
                <ImageCircledText src="./heart.svg" number={props.life} />
            </div>
        </ArcherElement>
    )
} 
function PlayerZoneBox(props:{
    src:string,
    number:number,
    rotate?:boolean,
    scale?:number
}) {
    return <div className="life-total-box">
        <ImageCircledText src={props.src} number={props.number} rotate={props.rotate} scale={props.scale} />
    </div>
}
function PlayerZoneSizes(props:{
    player:PlayerView
}) {
    return (
        <div className="player-zone-sizes">
            <PlayerZoneBox number={props.player.library.length} src={"./cardback.svg"} rotate={true} scale={127} />
            <PlayerZoneBox number={props.player.hand.length} src={"./hand.svg"} />
            <PlayerZoneBox number={props.player.graveyard.length} src={"./cardback.svg"} rotate={true} scale={127} />
        </div>
    );
}
type PlayerProps={
    game:GameState,
    player:PlayerView,
    actions:ActionsUnion,
    handlers:any,
    player_id:PlayerId
};
function PlayerBox(props:PlayerProps) {
    return (
        <div className="custom-height-flexbox" style={{ height: "100%", background: "lightgrey" }}>
            <LifeTotalBox life={props.player.life}
                player_id={props.player_id}
                actions={props.actions}
                handlers={props.handlers} />
            <div className="player-ui-bottom">       
                <ManaSymbols
                    game={props.game}
                    player={props.player}
                />
                <PlayerZoneSizes
                    player={props.player}
                />
            </div>
        </div>
    )
}

function HandAndBattlefield(props:PlayerProps) {
    const controlled = props.game.battlefield.filter((card_id) => {
        const card = props.game.cards[card_id];
        if (!card) {
            return false;
        }
        return card.controller === props.player_id;
    });
    return (
        <div className="hand-and-battlefield">
            <div className="hand">
                {props.player.hand.map((card_id) =>
                    <Card
                        game={props.game}
                        id={card_id}
                        key={card_id}
                        actions={props.actions}
                        handlers={props.handlers} />
                )}
            </div>
            <div className="battlefield">
                {controlled.map((card_id) =>
                    <Card
                        game={props.game}
                        id={card_id}
                        key={card_id}
                        actions={props.actions}
                        handlers={props.handlers} />
                )}
            </div>
        </div>
    )
}
function PlayerBoxes(props:{
    game:GameState,
    width:number,
    actions:ActionsUnion,
    handlers:any
}) {
    let player_entries = Object.entries(props.game.players);
    let style = { height: 100 / player_entries.length + "%" };
    return (
        <div className="vertical-flexbox" style={{ flexGrow: 1 }}>
            {player_entries.map(([player_id, player]) =>
                <div className="per-player" style={style} key={player_id}>
                    <div className="vertical-flexbox" style={{ width: props.width + "px" }}>
                        <PlayerBox
                            game={props.game}
                            player={player}
                            player_id={parseInt(player_id)}
                            key={player_id}
                            actions={props.actions}
                            handlers={props.handlers} />
                    </div>
                    <HandAndBattlefield
                        game={props.game}
                        player={player}
                        player_id={parseInt(player_id)}
                        key={player_id}
                        actions={props.actions}
                        handlers={props.handlers} />
                </div>
            )}
        </div>
    )
}

function process_actions(action_data: AskSelectNFor_Action): CardActions {
    const card_actions: CardActions = {};
    console.log(JSON.stringify(action_data.ents));
    action_data.ents.map((action_in, index) => {
        let id = null;
        let action: Action=(action_in as unknown as Action);//This is an action,
        //but due to how 
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
class Game extends React.Component<IProps, IState>{
    socket: WebSocket;
    constructor(props: {}) {
        super(props);
        const socket = new WebSocket("ws://localhost:3030/gamesetup");
        this.socket = socket;
        socket.addEventListener('message', (event: { data: string; }) => {
            let parsed: ClientMessage= JSON.parse(event.data);
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
        if (this.state.actions.type === "action") {
            this.socket.send("[]");
            this.clear_actions();
        }
        if (this.state.actions.type === "attackers") {
            const resp = this.state.actions.response.map(x => {
                if (x === null) {
                    return [];
                } else {
                    return [x];
                }
            });
            const response = JSON.stringify(resp);
            this.socket.send(response);
            this.clear_actions();
        }

    }
    componentDidMount() {
        document.addEventListener("keydown", this.keyPressed.bind(this), false);
    }
    componentWillUnmount() {
        document.removeEventListener("keydown", this.keyPressed.bind(this), false);
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
            const parsed_actions:AskSelectNFor_Action=parsed.Action;
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
            if (attackers.length === 0) {
                this.socket.send("[]");
                return;
            }
            actions = {
                type: "attackers",
                attackers: attackers,
                response: attackers.map((_i:number) => null),
                selected_attacker: null,
                targets: parsed.Attackers.b
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
            const attacker_index = actions.attackers.indexOf(ent_id);
            if (attacker_index !== -1) {
                if (actions.selected_attacker !== null) {
                    actions.selected_attacker = null;
                } else {
                    actions.selected_attacker = ent_id;
                }
            }
            const selected = actions.selected_attacker;
            if (selected !== null) {
                const target_index = actions.targets.indexOf(ent_id);
                if (target_index !== -1) {
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
    }
    render() {
        if (this.state.game) {
            return (
                <ArcherContainer strokeColor="red">
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
