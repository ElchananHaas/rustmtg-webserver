import React from 'react';
import { createRoot } from 'react-dom/client';
import { Stage, Layer, Circle } from 'react-konva';
import {PhaseImage, PHASE_IMAGE_KEY, PHASE_IMAGE_URL}  from "./phases.js";
class Game extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      width:600, //These should be overridden in the componentDidMount function
      height:400,
    };
    const socket = new WebSocket('ws://localhost:3030/gamesetup');
    this.state.socket=socket;
    let self=this;
    socket.addEventListener('message', function (event) {
      let parsed=JSON.parse(event.data);
      console.log(parsed);
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

  }
  update_state(parsed){
    this.setState({
      me:parsed[1],
      ecs:parsed[2],
      players:parsed[3],
      globals:parsed[4],
    });
  }
  respond_action(parsed){
    //TODO
  }
  componentDidMount() {
    this.checkSize();
    // here we should add listener for "container" resize
    // take a look here https://developers.google.com/web/updates/2016/10/resizeobserver
    // for simplicity I will just listen window resize
    window.addEventListener("resize", this.checkSize.bind(this));
  }
  componentWillUnmount() {
    window.removeEventListener("resize", this.checkSize.bind(this));
  }
  checkSize(){
    this.setState({
      width: window.innerWidth-10,
      height: window.innerHeight-10,
    });
  };
  render() {
      const phase_image_dist=this.state.height/PHASE_IMAGE_KEY.length;
      const phase_image_size=phase_image_dist*.9;
      return (
      // Stage - is a div wrapper
      // Layer - is an actual 2d canvas element, so you can have several layers inside the stage
      // Rect and Circle are not DOM elements. They are 2d shapes on canvas
      <Stage width={window.innerWidth} height={window.innerHeight}>
        <Layer>
          {
            PHASE_IMAGE_KEY.map((phase,i)=>(
              <PhaseImage
                key={phase}
                src={PHASE_IMAGE_URL[phase]}  
                y={phase_image_dist*Number(i)}
                size={phase_image_size}
              />
            ))
          }
        </Layer>
      </Stage>
    );
  }
} 

const container = document.getElementById('root');
const root = createRoot(container);
root.render(<Game />);