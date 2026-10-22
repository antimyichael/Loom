const projectPathInput = document.getElementById('projectPathInput');
const browseProjectBtn = document.getElementById('browseProjectBtn');
const dropZone = document.getElementById('dropZone');

const vaultNameInput = document.getElementById('vaultNameInput');
const vaultPathDisplay = document.getElementById('vaultPathDisplay');
const vaultPathInput = document.getElementById('vaultPathInput');
const browseVaultBtn = document.getElementById('browseVaultBtn');

const unityToggle = document.getElementById('unityToggle');
const unityDetectedBadge = document.getElementById('unityDetectedBadge');
const watchToggle = document.getElementById('watchToggle');
const verboseToggle = document.getElementById('verboseToggle');

const runBtn = document.getElementById('runBtn');
const runBtnText = document.getElementById('runBtnText');
const stopBtn = document.getElementById('stopBtn');
const openVaultBtn = document.getElementById('openVaultBtn');

const statsSection = document.getElementById('statsSection');
const statFiles = document.getElementById('statFiles');
const statFileNotes = document.getElementById('statFileNotes');
const statSymbolNotes = document.getElementById('statSymbolNotes');
const unityStatCard = document.getElementById('unityStatCard');
const statUnityAssets = document.getElementById('statUnityAssets');

const statusDot = document.getElementById('statusDot');
const statusText = document.getElementById('statusText');
const consoleOutput = document.getElementById('consoleOutput');
const clearLogsBtn = document.getElementById('clearLogsBtn');

let isRunning = false;
let currentProjectPath = '';
let currentVaultPath = '';
let isCustomVaultPath = false;

function stripAnsi(text) {
  return text.replace(/[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]/g, '');
}

function updateVaultDestination() {
  if (!currentProjectPath) return;
  const sep = currentProjectPath.includes('\\') ? '\\' : '/';
  const name = vaultNameInput.value.trim() || 'LoomVault';
  if (vaultPathDisplay) {
    vaultPathDisplay.textContent = name;
  }
  if (!isCustomVaultPath) {
    currentVaultPath = `${currentProjectPath}${sep}${name}`;
    vaultPathInput.value = currentVaultPath;
  }
}

vaultNameInput.addEventListener('input', () => {
  isCustomVaultPath = false;
  updateVaultDestination();
});

async function setProjectPath(dirPath) {
  if (!dirPath) return;
  currentProjectPath = dirPath;
  projectPathInput.value = dirPath;

  isCustomVaultPath = false;
  updateVaultDestination();

  runBtn.disabled = false;
  openVaultBtn.classList.add('hidden');

  try {
    const isUnity = await window.loomAPI.checkUnityProject(dirPath);
    if (isUnity) {
      unityDetectedBadge.classList.remove('hidden');
      unityToggle.checked = true;
      setStatus('Unity project detected', 'idle');
    } else {
      unityDetectedBadge.classList.add('hidden');
      setStatus('Ready to weave', 'idle');
    }
  } catch {
    unityDetectedBadge.classList.add('hidden');
  }
}

browseProjectBtn.addEventListener('click', async () => {
  const selected = await window.loomAPI.selectDirectory('Select Codebase Folder');
  if (selected) {
    await setProjectPath(selected);
  }
});

browseVaultBtn.addEventListener('click', async () => {
  const selected = await window.loomAPI.selectDirectory('Select Custom Vault Output Directory');
  if (selected) {
    isCustomVaultPath = true;
    currentVaultPath = selected;
    vaultPathInput.value = selected;
  }
});

['dragenter', 'dragover'].forEach(eventName => {
  window.addEventListener(eventName, (e) => {
    e.preventDefault();
    e.stopPropagation();
    dropZone.classList.add('dragover');
  });
});

['dragleave', 'drop'].forEach(eventName => {
  window.addEventListener(eventName, (e) => {
    e.preventDefault();
    e.stopPropagation();
    dropZone.classList.remove('dragover');
  });
});

window.addEventListener('drop', async (e) => {
  if (e.dataTransfer && e.dataTransfer.files.length > 0) {
    const droppedFile = e.dataTransfer.files[0];
    if (droppedFile.path) {
      await setProjectPath(droppedFile.path);
    }
  }
});

dropZone.addEventListener('click', () => {
  browseProjectBtn.click();
});

function setStatus(text, state) {
  statusText.textContent = text;
  statusDot.className = 'status-dot';
  if (state === 'active') {
    statusDot.classList.add('active');
  } else if (state === 'error') {
    statusDot.classList.add('error');
  }
}

runBtn.addEventListener('click', async () => {
  if (isRunning) return;

  if (!currentProjectPath) {
    alert('Please select a project repository directory first.');
    return;
  }

  isRunning = true;
  runBtn.disabled = true;
  runBtnText.textContent = watchToggle.checked ? 'Watching...' : 'Weaving...';

  if (watchToggle.checked) {
    stopBtn.classList.remove('hidden');
  }

  consoleOutput.innerHTML = '';
  statsSection.classList.remove('hidden');
  statFiles.textContent = '0';
  statFileNotes.textContent = '0';
  statSymbolNotes.textContent = '0';
  statUnityAssets.textContent = '0';

  if (unityToggle.checked) {
    unityStatCard.classList.remove('hidden');
  } else {
    unityStatCard.classList.add('hidden');
  }

  setStatus('Weaving vault...', 'active');

  const options = {
    projectPath: currentProjectPath,
    outputPath: isCustomVaultPath ? vaultPathInput.value.trim() : '',
    name: vaultNameInput.value.trim() || 'LoomVault',
    unity: unityToggle.checked,
    watch: watchToggle.checked,
    verbose: verboseToggle.checked,
  };

  const res = await window.loomAPI.runLoom(options);
  if (!res.success) {
    isRunning = false;
    runBtn.disabled = false;
    runBtnText.textContent = 'Weave Vault';
    stopBtn.classList.add('hidden');
    setStatus('Failed to start', 'error');
    appendLog(`Error: ${res.error}\n`);
  }
});

stopBtn.addEventListener('click', async () => {
  setStatus('Stopping watcher...', 'active');
  await window.loomAPI.stopLoom();
});

openVaultBtn.addEventListener('click', async () => {
  const targetVault = vaultPathInput.value || currentVaultPath;
  await window.loomAPI.openVault(targetVault);
});

clearLogsBtn.addEventListener('click', () => {
  consoleOutput.innerHTML = '';
});

function appendLog(rawText) {
  const clean = stripAnsi(rawText);

  const foundFilesMatch = clean.match(/Found (\d+) files to parse/);
  if (foundFilesMatch) {
    statFiles.textContent = foundFilesMatch[1];
  }

  const filesProcessedMatch = clean.match(/Files processed:\s+(\d+)/) || clean.match(/Source files processed:\s+(\d+)/);
  if (filesProcessedMatch) {
    statFiles.textContent = filesProcessedMatch[1];
  }

  const fileNotesMatch = clean.match(/File notes written:\s+(\d+)/);
  if (fileNotesMatch) {
    statFileNotes.textContent = fileNotesMatch[1];
  }

  const symbolNotesMatch = clean.match(/Symbol notes written:\s+(\d+)/);
  if (symbolNotesMatch) {
    statSymbolNotes.textContent = symbolNotesMatch[1];
  }

  const unityAssetsMatch = clean.match(/Indexed (\d+) Unity assets/);
  if (unityAssetsMatch) {
    unityStatCard.classList.remove('hidden');
    statUnityAssets.textContent = unityAssetsMatch[1];
  }

  const textNode = document.createTextNode(clean);
  consoleOutput.appendChild(textNode);
  consoleOutput.scrollTop = consoleOutput.scrollHeight;
}

window.loomAPI.onLog((data) => {
  appendLog(data.text);
});

window.loomAPI.onDone(({ code, error }) => {
  isRunning = false;
  runBtn.disabled = false;
  runBtnText.textContent = 'Weave Vault';
  stopBtn.classList.add('hidden');

  if (code === 0) {
    setStatus('Vault weaving complete!', 'idle');
    openVaultBtn.classList.remove('hidden');
  } else {
    setStatus(`Execution ended (code ${code})`, 'error');
  }

  if (error) {
    appendLog(`\nProcess error: ${error}\n`);
  }
});
