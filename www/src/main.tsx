import { App } from './App';
import './style.css'
import {render} from "solid-js/web";

const app = document.querySelector<HTMLDivElement>('#app')!

render(App, app);
