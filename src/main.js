import { invoke } from "@tauri-apps/api/core";

const list = document.querySelector("#list");
const editor = document.querySelector("#editor");
const form = document.querySelector("#form");
const error = document.querySelector("#error");
let actions = [];
let editingId = null;

const newId = () => `action-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
const platformLabel = (action) => action.platforms?.[0] || "All platforms";
const escapeHtml = (value) => { const element = document.createElement("div"); element.textContent = value; return element.innerHTML; };

function render() {
  if (!actions.length) { list.innerHTML = `<p class="empty">No actions yet.</p>`; return; }
  list.innerHTML = actions.map((action) => `<div class="action"><div class="action-name">${escapeHtml(action.label)}<div class="action-meta">${platformLabel(action)}</div></div><button class="secondary" data-edit="${action.id}">Edit</button><button class="secondary danger" data-delete="${action.id}">Delete</button></div>`).join("");
}
async function persist() { await invoke("save_actions", { actions }); }
function openEditor(action) {
  editingId = action?.id || null; error.textContent = "";
  document.querySelector("#editor-title").textContent = action ? "Edit action" : "Add action";
  document.querySelector("#label").value = action?.label || "";
  document.querySelector("#command").value = action?.command || "";
  document.querySelector("#directory").value = action?.working_directory || "";
  document.querySelector("#platform").value = action?.platforms?.[0] || "";
  editor.showModal(); document.querySelector("#label").focus();
}

document.querySelector("#add").addEventListener("click", () => openEditor());
document.querySelector("#cancel").addEventListener("click", () => editor.close());
list.addEventListener("click", async (event) => {
  const id = event.target.dataset.edit || event.target.dataset.delete;
  if (!id) return;
  const action = actions.find((item) => item.id === id);
  if (event.target.dataset.edit) openEditor(action);
  if (event.target.dataset.delete && confirm(`Delete “${action.label}”?`)) { actions = actions.filter((item) => item.id !== id); await persist(); render(); }
});
form.addEventListener("submit", async (event) => {
  event.preventDefault(); error.textContent = "";
  const label = document.querySelector("#label").value.trim(); const command = document.querySelector("#command").value.trim();
  if (!label || !command) { error.textContent = "Name and command are required."; return; }
  const directory = document.querySelector("#directory").value.trim(); const platform = document.querySelector("#platform").value;
  const action = { id: editingId || newId(), label, command, platforms: platform ? [platform] : [], ...(directory && { working_directory: directory }) };
  actions = editingId ? actions.map((item) => item.id === editingId ? action : item) : [...actions, action];
  try { await persist(); editor.close(); render(); } catch (reason) { error.textContent = String(reason); }
});

actions = await invoke("get_actions"); render();
