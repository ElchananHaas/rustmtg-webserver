import React from "react";
import { ArcherElement } from "react-archer";
import { RelationType } from "react-archer/lib/types";
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
    let style:React.CSSProperties={};
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
    const relation:RelationType={
        targetId: "",
        targetAnchor: 'bottom',
        sourceAnchor: 'top',
        style: { strokeColor: 'red', strokeWidth: 2 }
    };
    if(props.actions.type==="attackers"){
        if(props.id in props.actions.attackers){
            style.borderColor="#AA0000";
            const attacking=props.actions.response[props.id];
            if(attacking!==null){
                style.borderColor="#FF0000";
                relation.targetId=""+attacking;
            }
        }
        if(props.actions.selected_attacker===props.id){
            style.borderColor="#FF0000";
        }
    }
    const ret=(
        <ArcherElement id={""+props.id} relations={[relation]}>
            <div className="card-div" style={style} onClick={() => props.handlers.click(props.id)}>
                <img src={url} className="full-height-image" alt=""></img>
            </div>
        </ArcherElement>
    );
    return(ret);
  }