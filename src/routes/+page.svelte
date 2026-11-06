<script lang="ts">
  import { appState } from '$lib/store.svelte';
  import NavigationRail from '$lib/components/NavigationRail.svelte';
  import TopBar from '$lib/components/TopBar.svelte';
  import ModLibraryView from '$lib/components/ModLibraryView.svelte';
  import InstancesView from '$lib/components/InstancesView.svelte';
  import ProfilesView from '$lib/components/ProfilesView.svelte';
  import ModrinthView from '$lib/components/ModrinthView.svelte';
  import DiagnosticsView from '$lib/components/DiagnosticsView.svelte';
  import SettingsView from '$lib/components/SettingsView.svelte';
  import ModDetailDrawer from '$lib/components/ModDetailDrawer.svelte';
  import VirtualizationModal from '$lib/components/VirtualizationModal.svelte';
  import ToastContainer from '$lib/components/ToastContainer.svelte';

  $effect(() => {
    appState.init();
  });
</script>

<div class="flex h-screen w-screen overflow-hidden bg-[#09090b] text-zinc-100 font-sans select-none antialiased">
  <!-- Left Navigation Rail (Vortex Style) -->
  <NavigationRail />

  <!-- Main Application Content Area -->
  <div class="flex-1 flex flex-col min-w-0 h-full relative">
    <!-- Top Action Bar -->
    <TopBar />

    <!-- Active View Content -->
    <main class="flex-1 overflow-hidden relative">
      {#if appState.activeTab === 'mods'}
        <ModLibraryView />
      {:else if appState.activeTab === 'instances'}
        <InstancesView />
      {:else if appState.activeTab === 'profiles'}
        <ProfilesView />
      {:else if appState.activeTab === 'modrinth'}
        <ModrinthView />
      {:else if appState.activeTab === 'diagnostics'}
        <DiagnosticsView />
      {:else if appState.activeTab === 'settings'}
        <SettingsView />
      {/if}
    </main>

    <!-- Slide-over Drawer for Mod Details -->
    <ModDetailDrawer />

    <!-- Virtualization Preview & Confirmation Dialog -->
    <VirtualizationModal />

    <!-- Notification Toasts -->
    <ToastContainer />
  </div>
</div>
