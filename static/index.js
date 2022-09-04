import Game from "./game.js";
var config = {
    type: Phaser.AUTO,
    width: 1600,
    height: 700,
    parent: 'phaser-game',
    antialias: true,
    antialiasGL:true,
    scene: [
        Game
    ]
};

var game = new Phaser.Game(config);
