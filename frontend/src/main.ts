import { mount } from 'svelte'
import { setWorkerUrl } from 'maplibre-gl'
import workerUrl from 'maplibre-gl/dist/maplibre-gl-worker.mjs?worker&url'
import App from './App.svelte'
import 'maplibre-gl/dist/maplibre-gl.css'
import './app.css'

setWorkerUrl(workerUrl)
mount(App, { target: document.getElementById('app')! })
