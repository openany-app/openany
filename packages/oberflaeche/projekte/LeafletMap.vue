<script setup>
// Schlanker Leaflet-Wrapper für die Rubrik „Orte". Zwei Modi:
//  • interactive=false (Standard): winziger, nicht bedienbarer Kartenausschnitt
//    für die Orts-Karten in der Übersicht.
//  • interactive=true: Karte zum Pin-Setzen (Klick/Drag) im Anlege-Dialog,
//    zweiseitig über v-model an { lat, lng } gebunden.
// Kacheln kommen von den OpenStreetMap-Foundation-Servern (einzige externe
// Browser-Anfrage der App). Leaflet + Marker-Icons werden gebündelt – kein CDN.
import { ref, onMounted, onBeforeUnmount, watch, nextTick } from 'vue';
import L from 'leaflet';
import 'leaflet/dist/leaflet.css';
import markerIcon2x from 'leaflet/dist/images/marker-icon-2x.png';
import markerIcon from 'leaflet/dist/images/marker-icon.png';
import markerShadow from 'leaflet/dist/images/marker-shadow.png';

// Ein EIGENES Icon statt L.Icon.Default. Die Bilder müssen aus dem Bündel
// kommen, nicht von einem CDN (das blockt die CSP) – aber `Icon.Default`
// stellt seinem iconUrl immer noch seinen selbst erkannten `imagePath` voran
// (leaflet-src.js, Icon.Default._getIconUrl). Ein `mergeOptions` setzt nur
// das Ende, also entsteht ein doppelter Pfad.
//
// Dass das lange niemandem auffiel, liegt am Bauweg: marker-icon.png ist
// 1466 Bytes und damit unter Vites Einbettungsgrenze. Im Build steckt das
// Bild als data:-URI im CSS, Leaflets Erkennungs-Regex greift nicht mehr und
// liefert einen leeren imagePath – dort war die URL also zufällig richtig.
// Nur der Dev-Server bekam einen echten Pfad und damit
// „…/images//node_modules/leaflet/dist/images/marker-icon.png" – 404, und
// vom Pin blieb ein leerer 25×41-Rahmen stehen.
//
// Ein einfaches `L.icon()` benutzt Icon.prototype._getIconUrl, das nichts
// voranstellt. Die Maße sind die von Leaflet selbst.
const ORT_ICON = L.icon({
  iconUrl: markerIcon,
  iconRetinaUrl: markerIcon2x,
  shadowUrl: markerShadow,
  iconSize: [25, 41],
  iconAnchor: [12, 41],
  popupAnchor: [1, -34],
  tooltipAnchor: [16, -28],
  shadowSize: [41, 41],
});

const props = defineProps({
  modelValue: { type: Object, default: null }, // { lat, lng } | null
  interactive: { type: Boolean, default: false },
  zoom: { type: Number, default: 15 },
});
const emit = defineEmits(['update:modelValue']);

// Deutschland-Mittelpunkt als Startansicht, wenn noch kein Pin gesetzt ist.
const FALLBACK = { lat: 51.1657, lng: 10.4515, zoom: 5 };
// Zoomstufe „ganze Stadt": beim ersten Pin-Setzen wird höchstens bis
// hierhin gezoomt, damit die Orientierung erhalten bleibt.
const CITY_ZOOM = 10;

const el = ref(null);
let map = null;
let marker = null;

const placeMarker = (latlng) => {
  if (marker) {
    marker.setLatLng(latlng);
  } else {
    marker = L.marker(latlng, { icon: ORT_ICON, draggable: props.interactive }).addTo(map);
    if (props.interactive) {
      marker.on('dragend', () => {
        const p = marker.getLatLng();
        emit('update:modelValue', { lat: p.lat, lng: p.lng });
      });
    }
  }
};

onMounted(async () => {
  const hasValue = props.modelValue && Number.isFinite(props.modelValue.lat);
  const center = hasValue ? [props.modelValue.lat, props.modelValue.lng] : [FALLBACK.lat, FALLBACK.lng];
  const zoom = hasValue ? props.zoom : FALLBACK.zoom;

  map = L.map(el.value, {
    center,
    zoom,
    zoomControl: props.interactive,
    dragging: props.interactive,
    scrollWheelZoom: false,
    doubleClickZoom: props.interactive,
    boxZoom: props.interactive,
    touchZoom: props.interactive,
    keyboard: props.interactive,
    attributionControl: true,
  });

  L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
    maxZoom: 19,
    attribution: '&copy; OpenStreetMap',
  }).addTo(map);

  if (hasValue) placeMarker([props.modelValue.lat, props.modelValue.lng]);

  if (props.interactive) {
    map.on('click', (e) => {
      placeMarker(e.latlng);
      // Behutsam heranführen: aus der Übersicht nur bis auf Stadt-Ebene
      // zoomen (ganz Hamburg sichtbar), nie automatisch weiter hinein –
      // Feinjustage macht der Nutzer selbst per Zoom/Drag.
      if (map.getZoom() < CITY_ZOOM) map.setView(e.latlng, CITY_ZOOM);
      emit('update:modelValue', { lat: e.latlng.lat, lng: e.latlng.lng });
    });
  }

  // Container ist erst nach dem Layout korrekt vermessen.
  await nextTick();
  setTimeout(() => map && map.invalidateSize(), 60);
});

// Von aussen gesetzte Koordinaten (z. B. Adresssuche später) nachziehen.
// Im interaktiven Modus nur den Marker mitführen – die Ansicht steuert
// dort der Klick-Handler bzw. der Nutzer selbst (kein Zoomsprung).
watch(() => props.modelValue, (val) => {
  if (!map || !val || !Number.isFinite(val.lat)) return;
  placeMarker([val.lat, val.lng]);
  if (!props.interactive) map.setView([val.lat, val.lng], props.zoom);
});

onBeforeUnmount(() => {
  if (map) { map.remove(); map = null; marker = null; }
});
</script>

<template>
  <!-- isolate: kapselt Leaflets hohe z-index-Werte (Panes 400+), damit die
       Kartenkacheln nicht über Modals und Dialoge ragen. -->
  <div ref="el" class="leaflet-host isolate w-full h-full rounded-xl overflow-hidden bg-auflage"></div>
</template>

<style>
/* Attribution dezent halten; Karten-Panes erben die abgerundete Ecke. */
.leaflet-host .leaflet-control-attribution {
  font-size: 9px;
  background: rgba(255, 255, 255, 0.7);
}
</style>
