const sampleUsers = [
  {name:"Avery Morgan",email:"avery.morgan@example.com",created:"Oct 19, 2026",status:"Subscribed",plan:"Personal",term:"Annual",date:"Oct 19, 2027",initials:"AM",tone:""},
  {name:"Jordan Lee",email:"jordan.lee@example.com",created:"Oct 18, 2026",status:"Trial",plan:"Trial",term:"24-hour",date:"Today, 4:30 PM",initials:"JL",tone:"purple"},
  {name:"Sam Rivera",email:"sam.rivera@example.com",created:"Oct 17, 2026",status:"Payment issue",plan:"Personal",term:"Monthly",date:"Oct 21, 2026",initials:"SR",tone:"blue"},
  {name:"Taylor Quinn",email:"taylor.quinn@example.com",created:"Oct 16, 2026",status:"Subscribed",plan:"Personal",term:"Monthly",date:"Nov 16, 2026",initials:"TQ",tone:"orange"},
  {name:"Morgan Ellis",email:"morgan.ellis@example.com",created:"Oct 15, 2026",status:"Trial ended",plan:"—",term:"—",date:"Oct 14, 2026",initials:"ME",tone:"purple"},
  {name:"Riley Chen",email:"riley.chen@example.com",created:"Oct 14, 2026",status:"Subscribed",plan:"Personal",term:"Annual",date:"Oct 14, 2027",initials:"RC",tone:"blue"},
  {name:"Casey Brooks",email:"casey.brooks@example.com",created:"Oct 13, 2026",status:"Trial",plan:"Trial",term:"24-hour",date:"Oct 20, 2026",initials:"CB",tone:"orange"},
  {name:"Jamie Patel",email:"jamie.patel@example.com",created:"Oct 12, 2026",status:"Subscribed",plan:"Personal",term:"Monthly",date:"Nov 12, 2026",initials:"JP",tone:""}
];

const rows = document.querySelector("#user-rows");
const search = document.querySelector("#user-search");
const filter = document.querySelector("#status-filter");
const emptyState = document.querySelector("#empty-state");
const resultCount = document.querySelector("#result-count");

function statusClass(status) {
  if (status === "Subscribed") return "subscribed";
  if (status === "Trial") return "trial";
  if (status === "Trial ended") return "ended";
  return "issue";
}

function renderUsers() {
  const term = search.value.trim().toLowerCase();
  const status = filter.value;
  const visible = sampleUsers.filter((user) =>
    (status === "all" || user.status === status) &&
    `${user.name} ${user.email}`.toLowerCase().includes(term)
  );

  rows.replaceChildren(...visible.map((user) => {
    const row = document.createElement("tr");
    const accountCell = document.createElement("div");
    accountCell.className = "account-cell";
    const avatar = document.createElement("span");
    avatar.className = `user-avatar ${user.tone}`;
    avatar.textContent = user.initials;
    const identity = document.createElement("span");
    const name = document.createElement("strong");
    name.textContent = user.name;
    const email = document.createElement("small");
    email.textContent = user.email;
    identity.append(name, email);
    accountCell.append(avatar, identity);
    row.append(makeCell(accountCell), makeCell(user.created));

    const statusCell = document.createElement("td");
    const statusBadge = document.createElement("span");
    statusBadge.className = `status-badge ${statusClass(user.status)}`;
    statusBadge.textContent = user.status;
    statusCell.append(statusBadge);
    row.append(statusCell);

    const planCell = document.createElement("td");
    const plan = document.createElement("span");
    plan.className = "plan-name";
    plan.textContent = user.plan;
    const termLabel = document.createElement("span");
    termLabel.className = "plan-term";
    termLabel.textContent = ` ${user.term}`;
    planCell.append(plan, termLabel);
    row.append(planCell, makeCell(user.date));

    const actions = document.createElement("td");
    const menu = document.createElement("button");
    menu.type = "button";
    menu.className = "row-menu";
    menu.setAttribute("aria-label", `More options for ${user.name}`);
    menu.textContent = "···";
    actions.append(menu);
    row.append(actions);
    return row;
  }));
  emptyState.hidden = visible.length > 0;
  resultCount.textContent = `Showing ${visible.length} of ${sampleUsers.length} sample accounts`;
}

function makeCell(content) {
  const cell = document.createElement("td");
  if (content instanceof Node) cell.append(content);
  else cell.textContent = content;
  return cell;
}

search.addEventListener("input", renderUsers);
filter.addEventListener("change", renderUsers);
renderUsers();
