import DispCard from "./card.js";
export default class Game extends Phaser.Scene {
    constructor() {
        super({
            key: 'Game'
        });
        this.PHASE_GREY=0x808080;
        this.ICON_SIZE=25;
        this.BOX_SIZE=75;
    }
    
    preload() {
        this.self=this;
        this.canvas=this.sys.game.canvas;
        this.canvas.imageSmoothingEnabled = true;
        this.load.image('game_background', './background.jpg');
        this.load.image('card_back',"https://c1.scryfall.com/file/scryfall-card-backs/large/59/597b79b3-7d77-4261-871a-60dd17403388.jpg?1561757061");
        this.load.image('artifact_card',"./magic-m15-mse-style/acard.jpg")
        this.load.image('artifact_pt',"./magic-m15-mse-style/apt.png")
        this.load.image("White","./mana-master/svg/w.svg");
        this.load.image("Blue","./mana-master/svg/u.svg");
        this.load.image("Red","./mana-master/svg/r.svg");
        this.load.image("Green","./mana-master/svg/g.svg");
        this.load.image("Black","./mana-master/svg/b.svg");
        this.load.image("Untap","./phases/untap.svg");
        this.load.image("Upkeep","./phases/upkeep.svg");
        this.load.image("Draw","./phases/draw.svg");
        this.load.image("FirstMain","./phases/main1.svg");
        this.load.image("BeginCombat","./phases/combat_start.svg");
        this.load.image("Attackers","./phases/combat_attackers.svg");
        this.load.image("Blockers","./phases/combat_blockers.svg");
        this.load.image("Damage","./phases/combat_damage.svg");
        this.load.image("EndCombat","./phases/combat_end.svg");
        this.load.image("SecondMain","./phases/main2.svg");
        this.load.image("EndStep","./phases/cleanup.svg");
        this.load.image("Pass","./phases/pass.svg");
        //this.load.setBaseURL('http://labs.phaser.io');
    }

    create() {
        let self=this;
        let back=this.add.image(0,0, 'game_background');
        back.setDepth(-10000);
        const socket = new WebSocket('ws://localhost:3030/gamesetup');
        socket.addEventListener('message', function (event) {
            let parsed=JSON.parse(event.data);
            if(parsed[0]==="GameState"){
                self.update_state(parsed);
            }   
            if(parsed[0]==="Action"){
                self.respond_action(parsed);
            }
            if(parsed[0]==="Attackers"){
                self.choose_attackers(parsed);
            } 
        });
        this.socket=socket;
        this.space_response="None";
        var spaceBar = this.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.SPACE);
        spaceBar.on('down', function(event){
            if(self.space_response=="send_empty"){
                self.socket.send("[]");
            }
        });
        this.disp_cards={};
        const phase_images_key=[
            "Untap",
            "Upkeep",
            "Draw",
            "FirstMain",
            "BeginCombat",
            "Attackers",
            "Blockers",
            "Damage",
            "EndCombat",
            "SecondMain",
            "EndStep",
        ];
        this.phase_images={}
        for(let i=0;i<phase_images_key.length;i++){
            const phase_image=phase_images_key[i];
            const image=this.add.image(this.ICON_SIZE,
                this.ICON_SIZE*(1+2*i),phase_image)
            .setScale(.8)
            .setTint(this.PHASE_GREY);
            this.phase_images[phase_image]=image;
        }
    }

    width(){
        return this.canvas.width;
    }
    height(){
        return this.canvas.height;
    }
    phase_image_key(game_globals){
        const subphase=game_globals.subphase;
        if(subphase!=null){
            if(subphase=="FirstStrikeDamage"){
                return "Damage";
            }
            if(subphase=="Cleanup"){
                return "EndStep";
            }
            return subphase.toString();
        }
        if(game_globals.phase==null){
            return "Game not started yet";
        }
        if(game_globals.phase=="Combat"){
            return "BeginCombat";
        }
        return game_globals.phase.toString();
    }
    update_state(parsed){
        const me=parsed[1];
        const ecs=parsed[2];
        const players=parsed[3];
        const game_globals=parsed[4];
        console.log(ecs);
        const myplayer=players[me];
        const hand=myplayer["hand"];
        const game_mana=game_globals.mana.ents;
        this.ecs=ecs;
        const this_phase_key=this.phase_image_key(game_globals);
        for(const key in this.phase_images){
            this.phase_images[key].setTint(this.PHASE_GREY);
        }
        this.phase_images[this_phase_key].setTint(0xFFFFFF);
        for (let card in this.disp_cards){
            this.disp_cards[card].destroy();
        }
        this.disp_cards={};
        for (let i=0;i<hand.length;i++){
            this.add_disp_card(ecs,hand[i],150 + (i * 125), 500);
        }
        const battlefield=game_globals.battlefield;
        for(let i=0;i<battlefield.length;i++){
            this.add_disp_card(ecs,battlefield[i],150 + (i * 125), 300);
        }
        let i=0;
        const num_players=Object.keys(players).length;
        for(const player in players){
            if(player==me){
                continue;
            }
            const bounds={
                x_min:2*this.ICON_SIZE,
                x_max:this.width(),
                y_min:this.height()*i/num_players,
                y_max:(this.height()*(i+1))/num_players,
            }
            this.draw_player_board(player,bounds,game_globals);
            i+=1;
        }
        const bounds={
            x_min:2*this.ICON_SIZE,
            x_max:this.width(),
            y_min:this.height()*(num_players-1)/num_players,
            y_max:this.height(),
        }
        this.draw_player_board(me,bounds,game_globals);
    }
    draw_player_board(player,bounds,game_globals){
        console.log("drawing player board at " + JSON.stringify(bounds));
        let box_end=bounds.x_min+this.BOX_SIZE;
        const box_back = this.add.rectangle(bounds.x_min, bounds.y_min, box_end, bounds.y_max, 0x909090)
        .setDepth(-10).setOrigin(0,0).setStrokeStyle(4,0x000000);
    }
    add_disp_card(ecs,index,x,y){
        let ent=ecs[index];
        let disp_card = new DispCard(this,x,y, ent);
        this.add.existing(disp_card);
        this.disp_cards[index]=disp_card;
        return disp_card;
    }
    choose_attackers(parsed){
        this.clear_click_actions();
        let pair=parsed[1].PairAB;
        if(pair.a.length==0){
            this.socket.send(JSON.stringify([]));
        }
    }
    respond_action(parsed){
        this.clear_click_actions();
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
                if(ent.Cast!=null){
                    let disp_card=this.disp_cards[ent.Cast.source_card];
                    disp_card.click_actions.push(i);
                }
            }
            this.space_response="send_empty"
        }
    }
    clear_click_actions(){
        for(let card in this.disp_cards){
            this.disp_cards[card].click_actions=[];
        }
        this.space_response="None"
    }
    
    update() {
    
    }
}