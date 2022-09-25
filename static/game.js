const phase_image_map={
    "Untap":"./phases/untap.svg",
    "Upkeep":"./phases/upkeep.svg",
    "Draw":"./phases/draw.svg",
    "FirstMain":"./phases/main1.svg",
    "BeginCombat":"./phases/combat_start.svg",
    "Attackers":"./phases/combat_attackers.svg",
    "Blockers":"./phases/combat_blockers.svg",
    "Damage":"./phases/combat_damage.svg",
    "EndCombat":"./phases/combat_end.svg",
    "SecondMain":"./phases/main2.svg",
    "EndStep":"./phases/cleanup.svg",
    "Pass":"./phases/nextturn.svg",
};

const colors_map={
    "White":"./counters/w.svg",
    "Blue":"./counters/u.svg",
    "Red":"./counters/r.svg",
    "Green":"./counters/g.svg",
    "Black":"./counters/b.svg",
    "Colorless":"./counters/general.svg",
};

function phase_image_key(phase,subphase){
    if(subphase!=null){
        if(subphase=="FirstStrikeDamage"){
            return "Damage";
        }
        if(subphase=="Cleanup"){
            return "EndStep";
        }
        return subphase.toString();
    }
    if(phase==null){
        return "Untap";
    }
    if(phase=="Combat"){
        return "BeginCombat";
    }
    if(phase=="Begin"){
        return "Untap";
    }
    if(phase=="Ending"){
        return "EndStep";
    }
    return phase.toString();
}

function PhaseImage(props){
    return(<div className="full-size" style={{height:props.height+"%"}} >
        <img 
        src={props.src} 
        alt={props.name} 
        className="phase-image"></img>
        {props.darkened? <div className="darken" style={{height:props.height+"%"}}></div>:<div></div>}
        </div> );
}
function PhaseImages(props){
    const entries=Object.entries(props.phase_image_map);
    const current_phase=phase_image_key(props.phase,props.subphase);
    return (
        <div className="vertical-flexbox black-border">
            {entries.map(([key,url]) =>
                <PhaseImage 
                key={key}
                src={url}
                name={key}
                height={100.0/entries.length}
                darkened={key!=current_phase}
            />
            )}
        </div>
    );
}
function Stack(props){
    return(
        <div className="stack" style={{width:props.card_width+"px"}}>

        </div>
    )
}

function ImageCircledText(props){
    let rotate={};
    if(props.rotate){
        rotate = {
            transform: `rotate(90deg)`,
            width:"100%"
        };
    }
    if(props.scale){
        rotate.height=props.scale+"%";
        rotate.width=props.scale+"%";
    }
    return(<div className="vertical-flexbox"> 
        <img src={props.src} className="full-height-image" style={rotate}></img>
        <div className="circle-div">
        </div>
        <p className="text-over-content">
            {props.number}
        </p>
    </div>)
}
function LifeTotalBox(props){
    return <div className="life-total-box">
        <ImageCircledText src="./heart.svg" number={props.life}/>
    </div>
}
function ManaSymbol(props){
    return(<div className="custom-height-flexbox" style={{height:props.height+"%"}}>
            <img src={colors_map[props.color]} className="full-height-image"></img>
            <div className="centering-div">
                <p className="text-over-content">
                    {props.quantity}
                </p>
            </div>
    </div>);
}
function ManaSymbols(props){
    const colors=[
        "White",
        "Blue",
        "Black",
        "Red",
        "Green",
        "Colorless"
    ];
    const color_quantities={};
    colors.map(color => {color_quantities[color]=0;});
    const mana_pool=props.player.mana_pool;
    mana_pool.map(function(mana_id){
        color_quantities[props.game.globals.mana[mana_id].color]+=1;
    });
    return(
        colors.map(color =>
            <ManaSymbol key={color} color={color} quantity={color_quantities[color]} height={100/colors.length}/>
        )
    )
}
function PlayerZoneBox(props){
    return <div className="life-total-box">
        <ImageCircledText src={props.src} number={props.number} rotate={props.rotate} scale={props.scale}/>
    </div>
}
function PlayerZoneSizes(props){
    return(
        <div className="player-zone-sizes">
            <PlayerZoneBox number={props.player.library.length} src={"./cardback.svg"} rotate={true} scale={127}/>
            <PlayerZoneBox number={props.player.hand.length} src={"./hand.svg"}/>
            <PlayerZoneBox number={props.player.graveyard.length} src={"./cardback.svg"} rotate={true} scale={127}/>
        </div>
    );
}
function PlayerBox(props){
    return(
        <div className="custom-height-flexbox" style={{height:"100%",background:"lightgrey"}}>
            <LifeTotalBox life={props.player.life } name={props.player.name}/>
            <div className="player-ui-bottom">
                <div className="mana-symbols">
                    <ManaSymbols
                        game={props.game}
                        player={props.player} 
                        player_id={props.player_id} 
                    />
                </div>
                <PlayerZoneSizes
                    player={props.player} 
                />
            </div>
        </div>
    )
}
function Card(props){
    let card=props.game.ecs[props.id];
    if(!card){
        return null;
    }
    let url=null;
    if(card.name==""|| !card.art_url){
        url="./cardback.svg";
    } else{
        url=card.art_url;
    }
    return(
        <div style={{height:"100%"}} onClick={() => props.handlers.click(props.id)}>
            <img src={url} className="full-height-image"></img>
        </div>
    
    )
}
function HandAndBattlefield(props){
    return(
        <div className="hand-and-battlefield">
            <div className="hand">
                {props.player.hand.map((card_id)=>
                    <Card 
                    game={props.game} 
                    id={card_id}
                    key={card_id}
                    actions={props.actions}
                    handlers={props.handlers}/>
                )}
            </div>
        </div>
    )
}
function PlayerBoxes(props){
    let player_entries=Object.entries(props.game.players);
    let style={height:100/player_entries.length+"%"};
    return(
            <div className="vertical-flexbox" style={{flexGrow:1}}>
                {player_entries.map(([player_id,player]) =>
                    <div className="per-player" style={style} key={player_id}>
                        <div className="vertical-flexbox" style={{width:props.width+"px"}}>
                            <PlayerBox 
                            game={props.game}
                            player={player} 
                            player_id={player_id} 
                            key={player_id}/>
                        </div>
                        <HandAndBattlefield
                            game={props.game}
                            player={player} 
                            player_id={player_id} 
                            key={player_id}
                            actions={props.actions}
                            handlers={props.handlers}/>
                    </div>
                )}
            </div>
    )
}
function process_actions(action_data){
    const card_actions={};
    action_data.SelectN.ents.map((action, index) =>
    {
        let id=null;
        if(action.PlayLand){
            id=action.PlayLand;
        }
        if(action.Cast){
            id=action.Cast.source_card;
        }
        if(id==null){
            console.log("action is");
            console.log(action);
        }
        const data={card_id:id,action:action,index:index};
        if(card_actions[id]){
            card_actions[id].push(data);
        }
        else{
            card_actions[id]=[data];
        }
    });
    return card_actions;
}
class Game extends React.Component{
    constructor(props) {    
        super(props);    
        const socket = new WebSocket("ws://localhost:3030/gamesetup");
        this.socket=socket;
        socket.addEventListener('message', function (event) {
            let parsed=JSON.parse(event.data);
            console.log(parsed);
            if(parsed[0]==="GameState"){
                this.update_state(parsed);
            }   
            if(["Action","Attackers"].includes(parsed[0])){
                this.respond_action(parsed);
            }
        }.bind(this));
        this.state = {
            card_width:75,
            playerbox_width:125,
            game:null,
            handlers:{
                click:this.card_clicked.bind(this)
            },
            actions:{
                type:null,
                data:null,
            }
        }; 
    }
    update_state(parsed){
        const structures={
            me:parsed[1],
            ecs:parsed[2],
            players:parsed[3]};
        const state = Object.assign({}, structures, parsed[4]);
        this.clear_actions();
        this.setState({game:state});
    }
    respond_action(parsed){
        if(parsed[0]=="Action"){
            if(parsed[1].SelectN.ents.length==0){
                this.socket.send("[]");
                return;
            }
            parsed[1]=process_actions(parsed[1]);
        }
        this.setState({actions:{
            type:parsed[0],
            data:parsed[1],
        }});
    }
    clear_actions(){
        this.setState({
            actions:{
                type:null,
                data:null,
            }}
        );
    }
    card_clicked(card_id){
        if(this.state.actions.type=="Action"){
            const card_actions=this.state.actions.data[card_id];
            if(!card_actions){
                return;
            }
            if(card_actions.length==1){
                const to_send=JSON.stringify([card_actions[0].index]);
                this.socket.send(to_send);
                this.clear_actions();
            }
            else{
                throw "I don't know how to deal with multiple actions for one card yet!";
            }
        }
    }
    render() {
        if(this.state.game){
        return (
            <div className="full-size">
                <PhaseImages phase_image_map={phase_image_map} phase={this.state.game.phase} subphase={this.state.game.subphase}/>
                <Stack card_width={this.state.card_width}/>
                <PlayerBoxes game={this.state.game}
                 width={this.state.playerbox_width} 
                 actions={this.state.actions}
                 handlers={this.state.handlers}/>
            </div>
        );
        }else{
            return <p> Waiting for game to start</p>
        }
    }
}
ReactDOM.render(<Game />,document.getElementById("root"));