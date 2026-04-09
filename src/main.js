const { invoke } = window.__TAURI__?.core || window.__TAURI__.tauri;

document.addEventListener('DOMContentLoaded', () => {
  const btnConnect = document.getElementById('connect-btn');
  const btnDisconnect = document.getElementById('disconnect-btn');
  const statusDot = document.getElementById('status-dot');
  const statusText = document.getElementById('status-text');
  const connectionInput = document.getElementById('connection');
  const datalist = document.getElementById('profile_names');
  const btnDelete = document.getElementById('btn-delete-profile');
  
  let isConnected = false;
  let allProfiles = [];

  const updateUIState = (connected) => {
    isConnected = connected;
    if (connected) {
      statusDot.className = 'status-indicator connected';
      statusText.innerText = 'Connected';
      statusText.style.color = '#28a745';
      btnConnect.disabled = true;
      btnDisconnect.disabled = false;
    } else {
      statusDot.className = 'status-indicator disconnected';
      statusText.innerText = 'Disconnected';
      statusText.style.color = '#333';
      btnConnect.disabled = false;
      btnDisconnect.disabled = true;
    }
  };

  const renderProfiles = async () => {
    try {
      allProfiles = await invoke('get_profiles');
      datalist.innerHTML = '';
      allProfiles.forEach(p => {
        const opt = document.createElement('option');
        opt.value = p.name;
        datalist.appendChild(opt);
      });
    } catch (e) {
      console.error(e);
    }
  };

  connectionInput.addEventListener('change', () => {
    const val = connectionInput.value;
    const p = allProfiles.find(pr => pr.name === val);
    if (p) {
      document.getElementById('server').value = p.config.server;
      document.getElementById('username').value = p.config.username;
      document.getElementById('password').value = p.config.password || '';
      document.getElementById('save_password').checked = p.config.savePassword;
      document.getElementById('persistent').checked = p.config.persistent;
    }
  });

  btnDelete.onclick = async () => {
    const val = connectionInput.value;
    const p = allProfiles.find(pr => pr.name === val);
    if (p) {
      if (confirm(`Delete connection "${p.name}"?`)) {
        await invoke('delete_profile', { id: p.id });
        connectionInput.value = '';
        renderProfiles();
      }
    }
  };

  btnDisconnect.onclick = async () => {
    try {
      btnDisconnect.disabled = true;
      await invoke('toggle_vpn', { connect: false });
      updateUIState(false);
    } catch (e) {
      console.error(e);
      btnDisconnect.disabled = false;
    }
  };

  document.getElementById('vpn-form').addEventListener('submit', async (e) => {
    e.preventDefault();
    if (isConnected) return;
    
    statusDot.className = 'status-indicator connecting';
    statusText.innerText = 'Connecting...';
    statusText.style.color = '#0056b3';
    btnConnect.disabled = true;

    try {
      const config = {
        server: document.getElementById('server').value,
        username: document.getElementById('username').value,
        password: document.getElementById('password').value,
        savePassword: document.getElementById('save_password').checked,
        persistent: document.getElementById('persistent').checked
      };

      const name = connectionInput.value || config.server;
      const id = name.replace(/\s+/g, '_').toLowerCase();

      // Ensure profile is saved/updated on connect
      await invoke('save_profile', { profile: { id, name, config } });
      await renderProfiles();

      // Switch to this profile and run it
      await invoke('save_config', { config });
      await invoke('toggle_vpn', { connect: true });
      updateUIState(true);

    } catch (err) {
      console.error(err);
      alert('Error connecting: ' + err);
      updateUIState(false);
    }
  });

  let initializedUI = false;

  const fetchStatus = async () => {
    try {
      const status = await invoke('get_status');
      const isConnected = status.state === 'connected';
      updateUIState(isConnected);

      // If connected but UI doesn't know which profile
      if (isConnected && status.activeProfileId && !initializedUI) {
        const p = allProfiles.find(pr => pr.id === status.activeProfileId);
        if (p) {
          connectionInput.value = p.name;
          document.getElementById('server').value = p.config.server;
          document.getElementById('username').value = p.config.username;
          if (p.config.password) {
            document.getElementById('password').value = p.config.password;
          }
          document.getElementById('save_password').checked = p.config.savePassword;
          document.getElementById('persistent').checked = p.config.persistent;
        }
        initializedUI = true;
      }
    } catch (e) {
      console.error(e);
    }
  };

  fetchStatus();
  renderProfiles();
  
  // Status check loop
  setInterval(fetchStatus, 3000);
});
