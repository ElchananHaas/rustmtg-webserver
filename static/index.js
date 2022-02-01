import Game from "./game.js";
var config = {
    type: Phaser.AUTO,
    width: 1280,
    height: 600,
    parent: 'phaser-game',
    antialias: true,
    antialiasGL:true,
    scene: [
        Game
    ]
};

var game = new Phaser.Game(config);
