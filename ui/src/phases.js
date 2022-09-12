import React from 'react';
import { Image } from 'react-konva';
export const PHASE_IMAGE_KEY=[
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
export const PHASE_IMAGE_URL={
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
};

export class PhaseImage extends React.Component {
    constructor(props) {     
        super(props);   
        this.state = {
            image: null
        };
    }

    componentDidMount() {
      const image = new window.Image();
      image.src = this.props.src;
      image.onload = () => {
        // setState will redraw layer
        // because "image" property is changed
        this.setState({
          image: image
        });
      };
    }
  
    render() {
      return <Image 
      image={this.state.image} 
      x={this.props.x} 
      y={this.props.y} 
      height={this.props.size}
      width={this.props.size}/>;
    }
  }
  