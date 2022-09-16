

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

function PhaseImage(props){
    return( <img src={props.src} alt={props.name} style={{height:props.height+"%"}} className="phase-image"></img> );
}
function PhaseImages(props){
    let entries=Object.entries(props.phase_image_map);
    return (
        <div className="phase-image-box">
            {entries.map(([key,url]) =>
                <PhaseImage 
                key={key}
                src={url}
                name={key}
                height={100.0/entries.length}
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
function PlayerBoxes(props){
    let player_entries=Object.entries(props.game.players);
    return(
        <div className="player-boxes" style={{width:props.width+"px"}}>
            {player_entries.map(([player_id,player]) =>
                <PlayerBox 
                game={props.game}
                player={player} 
                player_id={player_id} 
                height={100/player_entries.length}
                key={player_id}/>
            )}
        </div>
    )
}
function NumberOverCircle(props){
    return(
        <div className="number-over-circle">
            <div className="circle-div">

            </div>
            <p className="circle-number"> {props.number}</p>
        </div>
    )
}
function ImageCircledText(props){
    <img src={props.src}className="life-total-img">
        <div className="inside-img">
            
        </div>
    </img>
}
function LifeTotalBox(props){
    return <div className="life-total-box">
        <img src="./heart.svg" className="life-total-img"></img>
        <NumberOverCircle life={props.life}/>
    </div>
}
function ManaSymbols(props){
    <div className="mana-symbols">

    </div>
}
function PlayerBox(props){
    return(
        <div className="player-box" style={{height:props.height+"%"}}>
            <LifeTotalBox life={props.player.life } name={props.player.name}/>
            <div className="player-box-column-split">

            </div>
        </div>
    )
}
class Game extends React.Component{
    constructor(props) {    
        super(props);    
        const socket = new WebSocket("ws://localhost:3030/gamesetup");
        socket.addEventListener('message', function (event) {
            let parsed=JSON.parse(event.data);
            console.log(parsed);
            if(parsed[0]==="GameState"){
                this.update_state(parsed);
            }   
            if(parsed[0]==="Action"){
                this.respond_action(parsed);
            }
            if(parsed[0]==="Attackers"){
                this.choose_attackers(parsed);
            } 
        }.bind(this));
        this.state = {
            card_width:75,
            playerbox_width:125,
        };  
    }
    update_state(parsed){
        const state={
            me:parsed[1],
            ecs:parsed[2],
            players:parsed[3],
            globals:parsed[4],
        };
        this.setState({game:state});
    }
    respond_action(parsed){

    }
    render() {
        return (
            <div className="game-root">
                <PhaseImages phase_image_map={phase_image_map}/>
                <Stack card_width={this.state.card_width}/>
                {this.state.game ? <PlayerBoxes game={this.state.game} width={this.state.playerbox_width}/> : <div></div>}
            </div>
        );
    }
}
ReactDOM.render(<Game />,document.getElementById("root"));