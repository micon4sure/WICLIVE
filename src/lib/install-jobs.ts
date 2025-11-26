import createJob, { Job } from './wic-job';
import axios from 'axios';
import { ref, reactive } from 'vue'
import { invoke } from '@tauri-apps/api';

const DEFAULT_INSTALL_DIR = 'C:\\Program Files (x86)\\Ubisoft\\World in Conflict'
const VANILLA_KEY = '3EXO-ELED-MXGY-FP5M-286R'
const SOVIET_KEY = 'LABG-U3MF-RG9G-95GB-AYTH'

const useInstallJobs = () => {
  const _installDir = ref(localStorage.getItem('install-dir') || DEFAULT_INSTALL_DIR)

  const _path_patch10 = ref('');
  const _path_patch11 = ref('');
  const _path_vcredist = ref('');
  const _path_zipped_hooks = ref('');

  // DOWNLOAD PATCH 10
  const download_patch10 = createJob('Download Patch 10', async () => {
    _path_patch10.value = await invoke('download_patch', { patch: 10 });
  }, 'download-patch');

  // DOWNLOAD PATCH 11
  const download_patch11 = createJob('Download Patch 11', async () => {
    _path_patch11.value = await invoke('download_patch', { patch: 11 });
  }, 'download-patch');

  // DOWNLOAD VCREDIST
  const download_vcredist = createJob('Download Visual Studio C++ Redistributable', async () => {
    _path_vcredist.value = await invoke('download_vcredist');
  }, 'download-vcredist');

  // INSTALL PATCH 10
  const install_patch10 = createJob('Install Patch 10', async () => {
    await invoke('install_patch', { installerPath: _path_patch10.value });
    // wait 3 seconds for the installer to wrap up
    await new Promise(resolve => setTimeout(resolve, 3000));
  });

  // INSTALL PATCH 11
  const install_patch11 = createJob('Install Patch 11', async () => {
    await invoke('install_patch', { installerPath: _path_patch11.value });
    // wait 3 seconds for the installer to wrap up
    await new Promise(resolve => setTimeout(resolve, 3000));
  });

  // INSTALL VCREDIST
  const install_vcredist = createJob('Install Visual Studio C++ Redistributable', async () => {
    await invoke('install_vcredist', { vcredistExe: _path_vcredist.value });
  });

  // SET CD KEY
  const set_cd_key = createJob('Set CD key', async () => {
    const soviet = await invoke('is_soviet_assault')
    const key = soviet ? SOVIET_KEY : VANILLA_KEY
    await invoke('set_cd_key', { key });
    const confirmKey = await invoke('get_cd_key')
    if (confirmKey !== key)
      throw new Error('CD key not set correctly')
  });

  // DOWNLOAD HOOKS
  const download_hooks = createJob('Download multiplayer fix', async () => {
    const latestHooks = await axios.get('https://www.wicgate.com/wic_cl_hook-version.txt')
    _path_zipped_hooks.value = await invoke('download_hooks', { version: latestHooks.data });
  }, 'download-hooks');

  // UNZIP HOOKS
  const unzip_hooks = createJob('Install multiplayer fix', async () => {
    await invoke('unzip_hooks', { zipPath: _path_zipped_hooks.value, installDir: _installDir.value });
  }, 'extract-hooks');

  // CREATE DESKTOP SHORTCUT
  const create_desktop_shortcut = createJob('Create desktop shortcut', async () => {
    await invoke('create_desktop_shortcut')
  });

  // CLEAN INSTALL DIRECTORY
  const clean_temp_directory_pre = createJob('Clean temp directory', async () => {
    await invoke('clean_temp_directory');
  });
  const clean_temp_directory_patches = createJob('Clean temp directory', async () => {
    await invoke('clean_temp_directory');
  });
  const clean_temp_directory_post = createJob('Clean temp directory', async () => {
    await invoke('clean_temp_directory');
  });

  // SET LAA FLAG
  const set_laa = createJob('Set Large Address Aware flag', async () => {
    await invoke('set_laa_flag')
    const confirmLAA = await invoke('get_laa_flag')
    if (!confirmLAA)
      throw new Error('LAA flag not set correctly')
  });

  return {
    _installDir,
    path_patch10: _path_patch10,
    path_patch11: _path_patch11,
    path_vcredist: _path_vcredist,
    path_zipped_hooks: _path_zipped_hooks,
    download_patch10,
    download_patch11,
    download_vcredist,
    install_patch10,
    install_patch11,
    install_vcredist,
    set_cd_key,
    set_laa,
    download_hooks,
    unzip_hooks,
    create_desktop_shortcut,
    clean_temp_directory_pre,
    clean_temp_directory_patches,
    clean_temp_directory_post,
  }
}

export default useInstallJobs;