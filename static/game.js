import DispCard from "./card.js";
export default class Game extends Phaser.Scene {
    constructor() {
        super({
            key: 'Game'
        });
    }

    preload() {
        this.self=this;
        this.canvas=this.sys.game.canvas;
        this.canvas.imageSmoothingEnabled = true;
        this.load.image('game_background', './background.jpg');
        this.load.image('card_back',"https://c1.scryfall.com/file/scryfall-card-backs/large/59/597b79b3-7d77-4261-871a-60dd17403388.jpg?1561757061");
        this.load.image('artifact_card',"./magic-m15-mse-style/acard.jpg")
        this.load.image('artifact_pt',"./magic-m15-mse-style/apt.png")
        //this.load.setBaseURL('http://labs.phaser.io');
    }
    width(){
        return this.canvas.width;
    }
    height(){
        return this.canvas.height;
    }
    update_state(parsed){
        let me=parsed[1];
        let ecs=parsed[2];
        let players=parsed[3];
        let game_globals=parsed[4];
        console.log(ecs);
        let myplayer=players[me];
        let hand=myplayer["hand"];
        this.ecs=ecs;
        for (let card in this.disp_cards){
            this.disp_cards[card].destroy();
        }
        this.disp_cards={};
        for (let i=0;i<hand.length;i++){
            this.add_disp_card(ecs,hand[i],150 + (i * 125), 500);
        }
        let gamestate=parsed[4];
        let battlefield=gamestate.battlefield;
        for(let i=0;i<battlefield.length;i++){
            this.add_disp_card(ecs,battlefield[i],150 + (i * 125), 300);
        }
        let phase_text="";
        if(game_globals.phase==null){
            phase_text="game not started yet";
        }else{
            phase_text=game_globals.phase.toString();
        }
        if(game_globals.subphase!=null){
            phase_text+=": "+game_globals.subphase.toString();
        }
        this.phase_text.setText(phase_text);
    }
    add_disp_card(ecs,index,x,y){
        let ent=ecs[index];
        let hand_card = new DispCard(this,x,y, ent);
        this.add.existing(hand_card);
        this.disp_cards[index]=hand_card;
    }
    respond_action(parsed){
        let choices=parsed[1].SelectN;
        let ents=choices.ents;
        this.action_ents=ents;
        console.log(choices);
        if(ents.length==0){
            this.socket.send("[]");
        }else{
            for(let i=0;i<ents.length;i++){
                let ent=ents[i];
                console.log(ent);
                if(ent.PlayLand!=null){
                    let disp_card=this.disp_cards[ent.PlayLand];
                    disp_card.click_actions.push(i);
                }
                if(ent.ActivateAbility!=null){
                    let disp_card=this.disp_cards[ent.ActivateAbility.source];
                    disp_card.click_actions.push(i);
                }
            }
            this.space_response="send_empty"
        }
    }
    clear_click_actions(){
        console.log(this.disp_cards)
        for(let card in this.disp_cards){
            this.disp_cards[card].click_actions=[];
        }
        this.space_response="None"
    }
    create() {
        let self=this;
        let back=this.add.image(0,0, 'game_background');
        back.setDepth(-10000);
        const socket = new WebSocket('ws://localhost:3030/gamesetup');
        socket.addEventListener('message', function (event) {
            event.data.text().then(
                function(json_text){
                    let parsed=JSON.parse(json_text);
                    console.log('parsed json is', parsed);
                    if(parsed[0]==="GameState"){
                        self.update_state(parsed);
                    }   
                    if(parsed[0]==="Action"){
                        self.respond_action(parsed);
                    }   
                }
            )
        });
        this.socket=socket;
        this.space_response="None";
        var spaceBar = this.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.SPACE);
        spaceBar.on('down', function(event){
            if(self.space_response=="send_empty"){
                this.socket.send("[]");
            }
        });
        this.disp_cards={};
        this.phase_text=this.add.text(500,50,"game not yet started");
    }
    
    update() {
    
    }
}