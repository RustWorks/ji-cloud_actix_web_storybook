import { LitElement, html, css, customElement, property } from 'lit-element';
import {classMap} from "lit-html/directives/class-map";
import {nothing} from "lit-html";
import {ThemeKind} from "@elements/module/_common/theme";
import {cardBackPath} from "@elements/module/memory/_common/helpers";

@customElement('main-card')
export class _ extends LitElement {
  static get styles() {
      return [css`

          section {
              transition: transform 0.8s;
              transform-style: preserve-3d;
          }

          :host([dragOver]) section.editing .front {
              border-style: dashed;
              border-radius: 16px;
              border-width: 3px;
              background-color: var(--light-blue-1);
          }

          section.editing .front {
              border-style: solid; 
              border-radius: 16px;
              border-width: 1px;
          }

          .front {
              background-color: white;
          }
          section, ::slotted(img-ji), .back > img-ui {
              width: 160px;
              height: 160px;
          }

          ::slotted(img-ui) {
              width: 56px;
              height: 56px;
          }

          ::slotted(img-ji), ::slotted(img-ui) {
                object-fit: contain;
            }

          section.flippable:hover {
              transform: rotateY(180deg);
          }

          .front, .back {
              justify-content: center;
              align-items: center;
              display: flex;
              position: absolute;
              width: 100%;
              height: 100%;
              -webkit-backface-visibility: hidden; /* Safari */
                  backface-visibility: hidden;
          }

              


          .back {
              transform: rotateY(180deg);
          }
            .back > img-ui {
                object-fit: cover;
            }
    `];
  }

  @property({type: Boolean, reflect: true})
  dragOver:boolean = false;

  @property({type: Boolean})
  flippable:boolean = false;

  @property()
  theme:ThemeKind= "";

  @property({type: Boolean})
  editing: boolean = false;

  render() {
      const {flippable, theme, editing} = this;

      const style = `border-color: var(--theme-${theme}-border-color)`;

      return html`
          <section class="${classMap({flippable, editing})}" >
          <div class="front" style="${style}"><slot></slot></div>
          <div class="back"><img-ui path="${cardBackPath(theme)}"></img-ui></div>
          </section>
      `
  }
}