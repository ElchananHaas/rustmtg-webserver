import HandCard from "./card.js";
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
        this.load.image('artifact_card',"./acard.jpg")
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
        console.log(ecs);
        let myplayer=ecs[me].player;
        let hand=myplayer["hand"];
        console.log(hand);
        for (let i=0;i<hand.length;i++){
            let ent=ecs[hand[i]];
            if (ent==null){ continue;}
            let hand_card = new HandCard(this,100 + (i * 150), 200, ent);
            this.add.existing(hand_card);
        }
    }
    create() {
        let self=this;
        this.add.image(0,0, 'game_background');
        const socket = new WebSocket('ws://localhost:3030/gamesetup');
        socket.addEventListener('message', function (event) {
            event.data.text().then(
                function(json_text){
                    let parsed=JSON.parse(json_text);
                    console.log('parsed json is', parsed);
                    if(parsed[0]==="GameState"){
                        self.update_state(parsed);
                    }   
                }
            )
        });
        this.socket=socket;
    
    }
    
    update() {
    
    }
}