<script setup>
/*
 * Einstellungen — dieselbe Seite wie in der Webapp, mit den Kacheln, die ohne
 * Konto Sinn ergeben: Design, Sprache, Start unter Speicher, Startseite. Dazu
 * die Kacheln, die es nur hier gibt: Matrix und E-Mail-Postfächer (dieses
 * Gerät meldet sich selbst an), Speicher und Abgleich.
 *
 * Was fehlt, fehlt mit Absicht: Profil, Passwort, App-Passwörter, die
 * Matrix-Anbindung über openany.de und Speicherstand gehören zu einem Konto
 * auf einer Instanz und stehen dort, in der Webapp. Den Adressbuch-Tausch
 * (vCard) gibt es seit Oktober 2026 nur hier: Das Adressbuch ist
 * verschlüsselt, die Webapp liest es nicht mehr.
 */
import { useI18n } from 'vue-i18n';
import { Settings as SettingsIcon } from 'lucide-vue-next';
import ModulePage from '@oberflaeche/base/ModulePage.vue';
import ModuleHeader from '@oberflaeche/base/ModuleHeader.vue';
import SettingsDesign from '@oberflaeche/einstellungen/SettingsDesign.vue';
import SettingsLanguage from '@oberflaeche/einstellungen/SettingsLanguage.vue';
import SettingsModules from '@oberflaeche/einstellungen/SettingsModules.vue';
import SettingsSpeicherStart from '@oberflaeche/einstellungen/SettingsSpeicherStart.vue';
import AbgleichKachel from '../einstellungen/AbgleichKachel.vue';
import SpeicherKachel from '../einstellungen/SpeicherKachel.vue';
import SicherungKachel from '../einstellungen/SicherungKachel.vue';
import AktualisierungKachel from '../einstellungen/AktualisierungKachel.vue';
import PostfachKachel from '../einstellungen/PostfachKachel.vue';
import MatrixKachel from '../einstellungen/MatrixKachel.vue';
import PgpKachel from '../einstellungen/PgpKachel.vue';
import AdressbuchKachel from '../einstellungen/AdressbuchKachel.vue';
import NachrichtenschluesselKachel from '../einstellungen/NachrichtenschluesselKachel.vue';
import { useStartModule } from '../startModule';

defineProps({ lage: Object });
const emit = defineEmits(['lage-geaendert']);

const { t } = useI18n();
const { auswahl, traegt, speichern } = useStartModule();
</script>

<template>
  <ModulePage>
    <ModuleHeader :icon="SettingsIcon">
      <span class="flex-1 min-w-0 text-sm sm:text-base font-extrabold text-schrift tracking-tight truncate">
        {{ t('settings.settings.title') }}
      </span>
    </ModuleHeader>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <SettingsDesign />
      <SettingsLanguage />
      <SettingsSpeicherStart />
      <SettingsModules :modules="auswahl" :auswahl="traegt" :speichern="speichern" />
      <MatrixKachel />
      <PostfachKachel />
      <PgpKachel />
      <AdressbuchKachel />
      <NachrichtenschluesselKachel @lage-geaendert="emit('lage-geaendert')" />
      <SpeicherKachel />
      <SicherungKachel />
      <AktualisierungKachel />
      <AbgleichKachel :lage="lage" @lage-geaendert="emit('lage-geaendert')" />
    </div>
  </ModulePage>
</template>
