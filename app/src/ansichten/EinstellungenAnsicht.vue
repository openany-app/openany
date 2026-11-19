<script setup>
/*
 * Einstellungen — dieselbe Seite wie in der Webapp, mit den Kacheln, die ohne
 * Konto Sinn ergeben: Design, Sprache, Start unter Speicher, Startseite. Dazu
 * die Kacheln, die es nur hier gibt: Matrix und E-Mail-Postfächer (dieses
 * Gerät meldet sich selbst an), Speicher und Abgleich. Das Wichtigste steht
 * oben, das Aussehen unten.
 *
 * Was fehlt, fehlt mit Absicht: Profil, App-Passwörter und Speicherstand
 * gehören zu einem Konto auf einer Instanz und stehen dort, in der Webapp.
 * Den Adressbuch-Tausch (vCard) gibt es nur hier: Das Adressbuch ist
 * verschlüsselt, die Webapp liest es nicht.
 */
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { Settings as SettingsIcon } from 'lucide-vue-next';
import ModulePage from '@oberflaeche/base/ModulePage.vue';
import ModuleHeader from '@oberflaeche/base/ModuleHeader.vue';
import SettingsDesign from '@oberflaeche/einstellungen/SettingsDesign.vue';
import SettingsLanguage from '@oberflaeche/einstellungen/SettingsLanguage.vue';
import SettingsModules from '@oberflaeche/einstellungen/SettingsModules.vue';
import SettingsSpeicherStart from '@oberflaeche/einstellungen/SettingsSpeicherStart.vue';
import AbgleichKachel from '../einstellungen/AbgleichKachel.vue';
import NahKachel from '../einstellungen/NahKachel.vue';
import SpeicherKachel from '../einstellungen/SpeicherKachel.vue';
import SicherungKachel from '../einstellungen/SicherungKachel.vue';
import AktualisierungKachel from '../einstellungen/AktualisierungKachel.vue';
import SignalKachel from '../einstellungen/SignalKachel.vue';
import PostfachKachel from '../einstellungen/PostfachKachel.vue';
import MatrixKachel from '../einstellungen/MatrixKachel.vue';
import PgpKachel from '../einstellungen/PgpKachel.vue';
import AdressbuchKachel from '../einstellungen/AdressbuchKachel.vue';
import KalenderKachel from '../einstellungen/KalenderKachel.vue';
import FotosAutomatischKachel from '../einstellungen/FotosAutomatischKachel.vue';
import BriefKachel from '../einstellungen/BriefKachel.vue';
import NachrichtenschluesselKachel from '../einstellungen/NachrichtenschluesselKachel.vue';
import MeldungenKachel from '../einstellungen/MeldungenKachel.vue';
import SicherheitsnummernKachel from '../einstellungen/SicherheitsnummernKachel.vue';
import EinstellungenVerzeichnis from '../einstellungen/EinstellungenVerzeichnis.vue';
import { useStartModule } from '../startModule';

defineProps({ lage: { type: Object, default: null } });
const emit = defineEmits(['lage-geaendert']);

const { t } = useI18n();
const { auswahl, traegt, speichern } = useStartModule();
// Das Element mit den Gruppen -- das Verzeichnis liest daraus, was da ist.
const gruppenWurzel = ref(null);
</script>

<template>
  <ModulePage>
    <ModuleHeader :icon="SettingsIcon">
      <span class="flex-1 min-w-0 text-sm sm:text-base font-extrabold text-schrift tracking-tight truncate">
        {{ t('settings.settings.title') }}
      </span>
    </ModuleHeader>

    <!-- Eine Gruppe, deren Kacheln alle nichts zeigen (die Aktualisierung unter
         Android), blendet sich mit ihrer Überschrift aus. -->
    <div class="grid grid-cols-1 lg:grid-cols-[18rem_1fr] gap-6 items-start">
    <EinstellungenVerzeichnis :wurzel="gruppenWurzel" />
    <div ref="gruppenWurzel" class="space-y-8 min-w-0">
      <section id="einstellungen-abgleich" data-gruppe="abgleich" :data-titel="t('app.einstellungenGruppe.abgleich')" class="hidden has-[.karte]:block scroll-mt-20">
        <h2 class="mb-3 inline-block px-3 py-1 rounded-full bg-flaeche border border-linie shadow-sm text-xs font-extrabold uppercase tracking-wider text-schrift">{{ t('app.einstellungenGruppe.abgleich') }}</h2>
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <AbgleichKachel :lage="lage" @lage-geaendert="emit('lage-geaendert')" />
          <NahKachel :lage="lage" @lage-geaendert="emit('lage-geaendert')" />
          <NachrichtenschluesselKachel class="lg:col-span-2" @lage-geaendert="emit('lage-geaendert')" />
          <SicherheitsnummernKachel class="lg:col-span-2" />
          <MeldungenKachel />
        </div>
      </section>
      <section id="einstellungen-speicher" data-gruppe="speicher" :data-titel="t('app.einstellungenGruppe.speicher')" class="hidden has-[.karte]:block scroll-mt-20">
        <h2 class="mb-3 inline-block px-3 py-1 rounded-full bg-flaeche border border-linie shadow-sm text-xs font-extrabold uppercase tracking-wider text-schrift">{{ t('app.einstellungenGruppe.speicher') }}</h2>
        <div class="grid grid-cols-1 gap-6">
          <SpeicherKachel />
          <SicherungKachel />
          <FotosAutomatischKachel />
        </div>
      </section>
      <section id="einstellungen-nachrichten" data-gruppe="nachrichten" :data-titel="t('app.einstellungenGruppe.nachrichten')" class="hidden has-[.karte]:block scroll-mt-20">
        <h2 class="mb-3 inline-block px-3 py-1 rounded-full bg-flaeche border border-linie shadow-sm text-xs font-extrabold uppercase tracking-wider text-schrift">{{ t('app.einstellungenGruppe.nachrichten') }}</h2>
        <div class="grid grid-cols-1 gap-6">
          <SignalKachel />
          <PostfachKachel />
          <PgpKachel />
          <MatrixKachel />
        </div>
      </section>
      <section id="einstellungen-adressbuch" data-gruppe="adressbuch" :data-titel="t('app.einstellungenGruppe.adressbuch')" class="hidden has-[.karte]:block scroll-mt-20">
        <h2 class="mb-3 inline-block px-3 py-1 rounded-full bg-flaeche border border-linie shadow-sm text-xs font-extrabold uppercase tracking-wider text-schrift">{{ t('app.einstellungenGruppe.adressbuch') }}</h2>
        <div class="grid grid-cols-1 gap-6">
          <KalenderKachel />
          <AdressbuchKachel />
          <BriefKachel />
        </div>
      </section>
      <section id="einstellungen-ansicht" data-gruppe="ansicht" :data-titel="t('app.einstellungenGruppe.ansicht')" class="hidden has-[.karte]:block scroll-mt-20">
        <h2 class="mb-3 inline-block px-3 py-1 rounded-full bg-flaeche border border-linie shadow-sm text-xs font-extrabold uppercase tracking-wider text-schrift">{{ t('app.einstellungenGruppe.ansicht') }}</h2>
        <div class="grid grid-cols-1 gap-6">
          <SettingsModules :modules="auswahl" :auswahl="traegt" :speichern="speichern" />
          <SettingsSpeicherStart />
          <SettingsDesign />
          <SettingsLanguage />
        </div>
      </section>
      <section id="einstellungen-programm" data-gruppe="programm" :data-titel="t('app.einstellungenGruppe.programm')" class="hidden has-[.karte]:block scroll-mt-20">
        <h2 class="mb-3 inline-block px-3 py-1 rounded-full bg-flaeche border border-linie shadow-sm text-xs font-extrabold uppercase tracking-wider text-schrift">{{ t('app.einstellungenGruppe.programm') }}</h2>
        <div class="grid grid-cols-1 gap-6">
          <AktualisierungKachel />
        </div>
      </section>
    </div>
    </div>
  </ModulePage>
</template>
