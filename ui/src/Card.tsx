import React from "react";
import { ActionsUnion } from "./actions";
import { CardId, GameState } from "./rustTypes";

export function Card(props:{
    game:GameState,
    id:CardId,
    actions:ActionsUnion,
    handlers:any
}){
    let card=props.game.cards[props.id];
    if(!card){
        return <div></div>;
    }
    let url=null;
    if(card.name===""|| !card.art_url){
        url="./cardback.svg";
    } else{
        url=card.art_url;
    }
    let style:any={};
    if(card.tapped){
        style = {
            transform: `rotate(15deg)`,
        };
    }
    if(props.actions.type==="action"){
        if(props.actions.actions[props.id] && props.actions.actions[props.id].length>0){
            style.borderColor="#AAAA00";
        }
    }
    if(props.actions.type==="attackers"){
        const index=props.actions.attackers.indexOf(props.id);
        console.log("attacker rendering");
        if(index!==-1){
            style.borderColor="#AA0000";
            if(props.actions.response[index]!==null){
                style.borderColor="#FF0000";
            }
        }
        if(props.actions.selected_attacker===props.id){
            style.borderColor="#FF0000";
        }
    }
    const ret=(
        <div className="card-div" style={style} onClick={() => props.handlers.click(props.id)}>
            <img src={url} className="full-height-image" alt=""></img>
        </div>
    );
    return(ret);
  }