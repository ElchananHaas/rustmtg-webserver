import React from "react";
import { ArcherElement } from "react-archer";
import { ActionsUnion } from "./actions";
import { Card } from "./Card";
import { ManaSymbols } from "./Mana";
import { GameState, PlayerId, PlayerView } from "./rustTypes";


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
    const response = props.actions.type === "attackers" && props.actions.targets.filter((attacked) => {
        return attacked === props.player_id;
    }).length > 0;
    if (response) {
        style.borderColor = "#AA0000";
    }
    return (
        <ArcherElement id={""+props.player_id}>
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
export function PlayerBoxes(props:{
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
