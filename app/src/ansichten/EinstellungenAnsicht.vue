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
import PostfachKachel from '../einstellungen/PostfachKachel.vue';
import MatrixKachel from '../einstellungen/MatrixKachel.vue';
import PgpKachel from '../einstellungen/PgpKachel.vue';
import AdressbuchKachel from '../einstellungen/AdressbuchKachel.vue';
import BriefKachel from '../einstellungen/BriefKachel.vue';
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

    <!-- Eine Gruppe, deren Kacheln alle nichts zeigen (die Aktualisierung unter
         Android), blendet sich mit ihrer Überschrift aus. -->
    <div class="space-y-8">
      <section class="hidden has-[.karte]:block">
        <h2 class="mb-3 px-1 text-xs font-extrabold uppercase tracking-wider text-leise">{{ t('app.einstellungenGruppe.abgleich') }}</h2>
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <AbgleichKachel :lage="lage" @lage-geaendert="emit('lage-geaendert')" />
          <NahKachel :lage="lage" @lage-geaendert="emit('lage-geaendert')" />
          <NachrichtenschluesselKachel class="lg:col-span-2" @lage-geaendert="emit('lage-geaendert')" />
        </div>
      </section>
      <section class="hidden has-[.karte]:block">
        <h2 class="mb-3 px-1 text-xs font-extrabold uppercase tracking-wider text-leise">{{ t('app.einstellungenGruppe.speicher') }}</h2>
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <SpeicherKachel />
          <SicherungKachel />
        </div>
      </section>
      <section class="hidden has-[.karte]:block">
        <h2 class="mb-3 px-1 text-xs font-extrabold uppercase tracking-wider text-leise">{{ t('app.einstellungenGruppe.nachrichten') }}</h2>
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <PostfachKachel />
          <PgpKachel />
          <MatrixKachel />
        </div>
      </section>
      <section class="hidden has-[.karte]:block">
        <h2 class="mb-3 px-1 text-xs font-extrabold uppercase tracking-wider text-leise">{{ t('app.einstellungenGruppe.adressbuch') }}</h2>
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <AdressbuchKachel />
          <BriefKachel />
        </div>
      </section>
      <section class="hidden has-[.karte]:block">
        <h2 class="mb-3 px-1 text-xs font-extrabold uppercase tracking-wider text-leise">{{ t('app.einstellungenGruppe.ansicht') }}</h2>
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <SettingsModules :modules="auswahl" :auswahl="traegt" :speichern="speichern" />
          <SettingsSpeicherStart />
          <SettingsDesign />
          <SettingsLanguage />
        </div>
      </section>
      <section class="hidden has-[.karte]:block">
        <h2 class="mb-3 px-1 text-xs font-extrabold uppercase tracking-wider text-leise">{{ t('app.einstellungenGruppe.programm') }}</h2>
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <AktualisierungKachel />
        </div>
      </section>
    </div>
  </ModulePage>
</template>
