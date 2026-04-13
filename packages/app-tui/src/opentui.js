import {
  BoxRenderable,
  InputRenderable,
  ScrollBoxRenderable,
  SelectRenderable,
  TabSelectRenderable,
  TextRenderable,
  createCliRenderer,
} from "@opentui/core";

const baseUrl = process.env.GUNMETAL_TUI_BASE_URL || "http://127.0.0.1:4684";

class GunmetalTuiApp {
  constructor() {
    this.renderer = null;
    this.data = null;
    this.status = "Loading Gunmetal...";
    this.selectedTab = "setup";
    this.selectedProfileId = null;
    this.selectedLogId = null;
    this.focusIndex = { setup: 0, playground: 0, requests: 0 };
    this.playground = {
      key: "",
      model: "",
      mode: "chat",
      historyMode: "conversation",
      messages: [],
      usage: null,
      durationMs: null,
      running: false,
    };
    this.requestFilters = {
      provider: "all",
      status: "all",
      query: "",
    };
    this.lastSecret = null;
    this.rendering = false;
    this.ui = {};
  }

  async start() {
    this.renderer = await createCliRenderer({ exitOnCtrlC: true });
    this.buildUi();
    this.bindKeys();
    await this.refreshState();
  }

  buildUi() {
    const renderer = this.renderer;
    const root = new BoxRenderable(renderer, {
      width: "100%",
      height: "100%",
      flexDirection: "column",
      padding: 1,
      gap: 1,
      backgroundColor: "#060708",
    });
    renderer.root.add(root);
    this.ui.root = root;

    const header = new BoxRenderable(renderer, {
      width: "100%",
      flexDirection: "column",
      border: true,
      borderStyle: "rounded",
      borderColor: "#3a414a",
      focusedBorderColor: "#d8b26e",
      padding: 1,
      gap: 1,
      backgroundColor: "#101418",
      title: "Gunmetal",
    });
    root.add(header);
    this.ui.headerTitle = new TextRenderable(renderer, {
      content: "Local inference. Terminal comfort.",
      textColor: "#f4f4ee",
    });
    this.ui.headerCounts = new TextRenderable(renderer, {
      content: "Loading counts...",
      textColor: "#9aa3ad",
    });
    this.ui.headerNext = new TextRenderable(renderer, {
      content: "Waiting for local state...",
      textColor: "#9aa3ad",
    });
    header.add(this.ui.headerTitle);
    header.add(this.ui.headerCounts);
    header.add(this.ui.headerNext);

    this.ui.tabs = new TabSelectRenderable(renderer, {
      width: "100%",
      border: true,
      borderStyle: "rounded",
      borderColor: "#3a414a",
      focusedBorderColor: "#7dd3c8",
      backgroundColor: "#101418",
      selectedBackgroundColor: "#d8b26e",
      selectedTextColor: "#060708",
      options: [
        { name: "Setup", description: "Connect provider, auth, sync, key" },
        { name: "Playground", description: "Chat through a Gunmetal key" },
        { name: "Requests", description: "Inspect logs and token stats" },
      ],
      selectedIndex: 0,
    });
    this.ui.tabs.on("selectionChanged", (_index, option) => {
      if (this.rendering) return;
      this.selectedTab = String(option?.value || option?.name || "setup").toLowerCase();
      this.syncVisibleView();
      this.focusCurrentView();
      this.renderAll();
    });
    root.add(this.ui.tabs);

    this.ui.content = new BoxRenderable(renderer, {
      width: "100%",
      flexGrow: 1,
      flexDirection: "column",
      gap: 1,
    });
    root.add(this.ui.content);

    this.buildSetupView();
    this.buildPlaygroundView();
    this.buildRequestsView();

    this.ui.statusLine = new TextRenderable(renderer, {
      content: this.status,
      textColor: "#f4f4ee",
    });
    root.add(this.ui.statusLine);

    this.ui.footer = new TextRenderable(renderer, {
      content: "Ctrl+1 setup  Ctrl+2 playground  Ctrl+3 requests  Ctrl+R refresh  Tab move focus  Ctrl+C quit",
      textColor: "#9aa3ad",
    });
    root.add(this.ui.footer);

    this.syncVisibleView();
  }

  buildSetupView() {
    const renderer = this.renderer;
    const view = new BoxRenderable(renderer, {
      width: "100%",
      flexGrow: 1,
      flexDirection: "row",
      gap: 1,
    });
    this.ui.content.add(view);
    this.ui.setupView = view;

    const providerColumn = new BoxRenderable(renderer, {
      width: "40%",
      height: "100%",
      flexDirection: "column",
      gap: 1,
      border: true,
      borderStyle: "rounded",
      borderColor: "#3a414a",
      focusedBorderColor: "#7dd3c8",
      padding: 1,
      title: "Providers",
      backgroundColor: "#101418",
    });
    view.add(providerColumn);

    this.ui.providerList = new SelectRenderable(renderer, {
      width: "100%",
      height: 12,
      border: true,
      borderStyle: "rounded",
      borderColor: "#24303a",
      focusedBorderColor: "#7dd3c8",
      selectedBackgroundColor: "#1f2c31",
      options: [],
      showDescription: true,
      wrapSelection: true,
    });
    this.ui.providerList.on("selectionChanged", (_index, option) => {
      if (this.rendering) return;
      this.selectedProfileId = option?.value || null;
      this.renderAll();
    });
    providerColumn.add(this.ui.providerList);

    this.ui.providerDetails = new TextRenderable(renderer, {
      width: "100%",
      flexGrow: 1,
      content: "No providers yet.",
      textColor: "#f4f4ee",
    });
    providerColumn.add(this.ui.providerDetails);

    const formColumn = new BoxRenderable(renderer, {
      width: "60%",
      height: "100%",
      flexDirection: "column",
      gap: 1,
      border: true,
      borderStyle: "rounded",
      borderColor: "#3a414a",
      focusedBorderColor: "#d8b26e",
      padding: 1,
      title: "Connect provider",
      backgroundColor: "#101418",
    });
    view.add(formColumn);

    this.ui.providerKindSelect = new SelectRenderable(renderer, {
      width: "100%",
      height: 8,
      border: true,
      borderStyle: "rounded",
      borderColor: "#24303a",
      focusedBorderColor: "#d8b26e",
      selectedBackgroundColor: "#2d2415",
      options: [],
      showDescription: true,
      wrapSelection: true,
    });
    this.ui.providerKindSelect.on("selectionChanged", () => {
      if (this.rendering) return;
      this.renderSetupFormMeta();
    });
    formColumn.add(this.ui.providerKindSelect);

    this.ui.providerNameInput = new InputRenderable(renderer, {
      width: "100%",
      border: true,
      borderStyle: "rounded",
      borderColor: "#24303a",
      focusedBorderColor: "#d8b26e",
      placeholder: "work-openai",
    });
    formColumn.add(this.ui.providerNameInput);

    this.ui.providerBaseUrlInput = new InputRenderable(renderer, {
      width: "100%",
      border: true,
      borderStyle: "rounded",
      borderColor: "#24303a",
      focusedBorderColor: "#d8b26e",
      placeholder: "optional base URL",
    });
    formColumn.add(this.ui.providerBaseUrlInput);

    this.ui.providerApiKeyInput = new InputRenderable(renderer, {
      width: "100%",
      border: true,
      borderStyle: "rounded",
      borderColor: "#24303a",
      focusedBorderColor: "#d8b26e",
      placeholder: "upstream API key",
    });
    formColumn.add(this.ui.providerApiKeyInput);

    this.ui.setupMeta = new TextRenderable(renderer, {
      width: "100%",
      flexGrow: 1,
      content: "Ctrl+S save  Ctrl+A auth  Ctrl+M sync models  Ctrl+K create key  Ctrl+O logout  Ctrl+D delete provider",
      textColor: "#9aa3ad",
    });
    formColumn.add(this.ui.setupMeta);
  }

  buildPlaygroundView() {
    const renderer = this.renderer;
    const view = new BoxRenderable(renderer, {
      width: "100%",
      flexGrow: 1,
      flexDirection: "column",
      gap: 1,
      border: true,
      borderStyle: "rounded",
      borderColor: "#3a414a",
      focusedBorderColor: "#7dd3c8",
      padding: 1,
      title: "Playground",
      backgroundColor: "#101418",
    });
    this.ui.content.add(view);
    this.ui.playgroundView = view;

    const toolbar = new BoxRenderable(renderer, {
      width: "100%",
      flexDirection: "row",
      gap: 1,
      height: 10,
    });
    view.add(toolbar);

    this.ui.playgroundKeyInput = new InputRenderable(renderer, {
      width: "38%",
      border: true,
      borderStyle: "rounded",
      borderColor: "#24303a",
      focusedBorderColor: "#d8b26e",
      placeholder: "Gunmetal key (gm_...)",
    });
    toolbar.add(this.ui.playgroundKeyInput);

    this.ui.playgroundModelSelect = new SelectRenderable(renderer, {
      width: "32%",
      height: 10,
      border: true,
      borderStyle: "rounded",
      borderColor: "#24303a",
      focusedBorderColor: "#7dd3c8",
      selectedBackgroundColor: "#1f2c31",
      options: [],
      showDescription: false,
    });
    this.ui.playgroundModelSelect.on("selectionChanged", (_index, option) => {
      if (this.rendering) return;
      this.playground.model = String(option?.value || "");
      this.renderAll();
    });
    toolbar.add(this.ui.playgroundModelSelect);

    const sideControls = new BoxRenderable(renderer, {
      width: "30%",
      height: 10,
      flexDirection: "column",
      gap: 1,
    });
    toolbar.add(sideControls);

    this.ui.playgroundModeSelect = new SelectRenderable(renderer, {
      width: "100%",
      height: 4,
      border: true,
      borderStyle: "rounded",
      borderColor: "#24303a",
      focusedBorderColor: "#7dd3c8",
      selectedBackgroundColor: "#1f2c31",
      options: [
        { name: "chat/completions", description: "OpenAI chat endpoint", value: "chat" },
        { name: "responses", description: "OpenAI responses endpoint", value: "responses" },
      ],
      showDescription: false,
    });
    this.ui.playgroundModeSelect.on("selectionChanged", (_index, option) => {
      if (this.rendering) return;
      this.playground.mode = String(option?.value || "chat");
      this.renderAll();
    });
    sideControls.add(this.ui.playgroundModeSelect);

    this.ui.playgroundHistorySelect = new SelectRenderable(renderer, {
      width: "100%",
      height: 4,
      border: true,
      borderStyle: "rounded",
      borderColor: "#24303a",
      focusedBorderColor: "#7dd3c8",
      selectedBackgroundColor: "#1f2c31",
      options: [
        { name: "keep conversation", description: "Send full transcript", value: "conversation" },
        { name: "single message", description: "Send only the last user prompt", value: "single" },
      ],
      showDescription: false,
    });
    this.ui.playgroundHistorySelect.on("selectionChanged", (_index, option) => {
      if (this.rendering) return;
      this.playground.historyMode = String(option?.value || "conversation");
      this.renderAll();
    });
    sideControls.add(this.ui.playgroundHistorySelect);

    this.ui.playgroundMeta = new TextRenderable(renderer, {
      width: "100%",
      content: "Enter sends. Ctrl+L clears the chat.",
      textColor: "#9aa3ad",
    });
    view.add(this.ui.playgroundMeta);

    this.ui.playgroundTranscript = new ScrollBoxRenderable(renderer, {
      width: "100%",
      flexGrow: 1,
      border: true,
      borderStyle: "rounded",
      borderColor: "#24303a",
      stickyScroll: true,
      stickyStart: "bottom",
      padding: 1,
      backgroundColor: "#0d1013",
    });
    view.add(this.ui.playgroundTranscript);

    this.ui.playgroundInput = new InputRenderable(renderer, {
      width: "100%",
      border: true,
      borderStyle: "rounded",
      borderColor: "#24303a",
      focusedBorderColor: "#d8b26e",
      placeholder: "Ask the selected model to say hello",
    });
    this.ui.playgroundInput.on("enter", async () => {
      await this.sendPlayground();
    });
    view.add(this.ui.playgroundInput);
  }

  buildRequestsView() {
    const renderer = this.renderer;
    const view = new BoxRenderable(renderer, {
      width: "100%",
      flexGrow: 1,
      flexDirection: "row",
      gap: 1,
    });
    this.ui.content.add(view);
    this.ui.requestsView = view;

    this.ui.requestList = new SelectRenderable(renderer, {
      width: "40%",
      height: "100%",
      border: true,
      borderStyle: "rounded",
      borderColor: "#3a414a",
      focusedBorderColor: "#7dd3c8",
      selectedBackgroundColor: "#1f2c31",
      options: [],
      showDescription: true,
      wrapSelection: true,
    });
    this.ui.requestList.on("selectionChanged", (_index, option) => {
      if (this.rendering) return;
      this.selectedLogId = option?.value || null;
      this.renderAll();
    });
    view.add(this.ui.requestList);

    const detailColumn = new BoxRenderable(renderer, {
      width: "60%",
      height: "100%",
      flexDirection: "column",
      gap: 1,
      border: true,
      borderStyle: "rounded",
      borderColor: "#3a414a",
      focusedBorderColor: "#d8b26e",
      padding: 1,
      title: "Request detail",
      backgroundColor: "#101418",
    });
    view.add(detailColumn);

    const filterRow = new BoxRenderable(renderer, {
      width: "100%",
      height: 8,
      flexDirection: "row",
      gap: 1,
    });
    detailColumn.add(filterRow);

    this.ui.requestProviderFilter = new SelectRenderable(renderer, {
      width: "50%",
      height: 8,
      border: true,
      borderStyle: "rounded",
      borderColor: "#24303a",
      focusedBorderColor: "#7dd3c8",
      selectedBackgroundColor: "#1f2c31",
      options: [],
      showDescription: false,
      wrapSelection: true,
    });
    this.ui.requestProviderFilter.on("selectionChanged", (_index, option) => {
      if (this.rendering) return;
      this.requestFilters.provider = String(option?.value || "all");
      this.renderAll();
    });
    filterRow.add(this.ui.requestProviderFilter);

    this.ui.requestStatusFilter = new SelectRenderable(renderer, {
      width: "50%",
      height: 8,
      border: true,
      borderStyle: "rounded",
      borderColor: "#24303a",
      focusedBorderColor: "#7dd3c8",
      selectedBackgroundColor: "#1f2c31",
      options: [
        { name: "all statuses", value: "all" },
        { name: "success only", value: "success" },
        { name: "errors only", value: "error" },
      ],
      showDescription: false,
      wrapSelection: true,
    });
    this.ui.requestStatusFilter.on("selectionChanged", (_index, option) => {
      if (this.rendering) return;
      this.requestFilters.status = String(option?.value || "all");
      this.renderAll();
    });
    filterRow.add(this.ui.requestStatusFilter);

    this.ui.requestQueryInput = new InputRenderable(renderer, {
      width: "100%",
      border: true,
      borderStyle: "rounded",
      borderColor: "#24303a",
      focusedBorderColor: "#7dd3c8",
      placeholder: "search model, key, endpoint, error",
    });
    this.ui.requestQueryInput.on("change", () => {
      if (this.rendering) return;
      this.requestFilters.query = this.ui.requestQueryInput.value.trim();
      this.renderAll();
    });
    detailColumn.add(this.ui.requestQueryInput);

    this.ui.requestTraffic = new TextRenderable(renderer, {
      width: "100%",
      content: "No traffic yet.",
      textColor: "#9aa3ad",
    });
    this.ui.requestSummary = new TextRenderable(renderer, {
      width: "100%",
      content: "Waiting for traffic summaries...",
      textColor: "#f4f4ee",
    });
    this.ui.requestDetails = new TextRenderable(renderer, {
      width: "100%",
      flexGrow: 1,
      content: "Use the playground or any OpenAI-compatible app and the first request will appear here.",
      textColor: "#f4f4ee",
    });
    detailColumn.add(this.ui.requestTraffic);
    detailColumn.add(this.ui.requestSummary);
    detailColumn.add(this.ui.requestDetails);
  }

  bindKeys() {
    this.renderer.keyInput.on("keypress", async (key) => {
      if (key.ctrl && key.name === "r") {
        key.preventDefault();
        await this.refreshState();
        return;
      }
      if (key.ctrl && key.name === "1") {
        key.preventDefault();
        this.selectTab(0);
        return;
      }
      if (key.ctrl && key.name === "2") {
        key.preventDefault();
        this.selectTab(1);
        return;
      }
      if (key.ctrl && key.name === "3") {
        key.preventDefault();
        this.selectTab(2);
        return;
      }
      if (key.name === "tab") {
        key.preventDefault();
        this.cycleFocus(key.shift ? -1 : 1);
        return;
      }

      if (this.selectedTab === "setup") {
        await this.handleSetupKeys(key);
      } else if (this.selectedTab === "playground") {
        await this.handlePlaygroundKeys(key);
      }
    });
  }

  async handleSetupKeys(key) {
    if (key.ctrl && key.name === "s") {
      key.preventDefault();
      await this.saveProvider();
    } else if (key.ctrl && key.name === "a") {
      key.preventDefault();
      await this.providerAction("auth");
    } else if (key.ctrl && key.name === "m") {
      key.preventDefault();
      await this.providerAction("sync");
    } else if (key.ctrl && key.name === "k") {
      key.preventDefault();
      await this.createKey();
    } else if (key.ctrl && key.name === "o") {
      key.preventDefault();
      await this.providerAction("logout");
    } else if (key.ctrl && key.name === "d") {
      key.preventDefault();
      await this.deleteProfile();
    }
  }

  async handlePlaygroundKeys(key) {
    if (key.ctrl && key.name === "l") {
      key.preventDefault();
      this.playground.messages = [];
      this.playground.usage = null;
      this.playground.durationMs = null;
      this.renderAll();
    }
  }

  selectTab(index) {
    this.ui.tabs.setSelectedIndex(index);
    const option = this.ui.tabs.getSelectedOption();
    this.selectedTab = String(option?.value || option?.name || "setup").toLowerCase();
    this.syncVisibleView();
    this.focusCurrentView();
    this.renderAll();
  }

  currentFocusables() {
    if (this.selectedTab === "playground") {
      return [
        this.ui.playgroundKeyInput,
        this.ui.playgroundModelSelect,
        this.ui.playgroundModeSelect,
        this.ui.playgroundHistorySelect,
        this.ui.playgroundInput,
      ];
    }
    if (this.selectedTab === "requests") {
      return [
        this.ui.requestList,
        this.ui.requestProviderFilter,
        this.ui.requestStatusFilter,
        this.ui.requestQueryInput,
      ];
    }

    const focusables = [
      this.ui.providerList,
      this.ui.providerKindSelect,
      this.ui.providerNameInput,
    ];
    if (this.ui.providerBaseUrlInput.visible) {
      focusables.push(this.ui.providerBaseUrlInput);
    }
    if (this.ui.providerApiKeyInput.visible) {
      focusables.push(this.ui.providerApiKeyInput);
    }
    return focusables;
  }

  cycleFocus(delta) {
    const focusables = this.currentFocusables().filter(Boolean);
    if (!focusables.length) {
      return;
    }
    const current = this.focusIndex[this.selectedTab];
    const next = (current + delta + focusables.length) % focusables.length;
    this.focusIndex[this.selectedTab] = next;
    focusables[next].focus();
  }

  focusCurrentView() {
    const focusables = this.currentFocusables().filter(Boolean);
    if (!focusables.length) {
      return;
    }
    const index = Math.min(this.focusIndex[this.selectedTab], focusables.length - 1);
    this.focusIndex[this.selectedTab] = index;
    focusables[index].focus();
  }

  syncVisibleView() {
    this.ui.setupView.visible = this.selectedTab === "setup";
    this.ui.playgroundView.visible = this.selectedTab === "playground";
    this.ui.requestsView.visible = this.selectedTab === "requests";
  }

  async refreshState() {
    try {
      this.data = await fetchJson("/app/api/state");
      this.status = "Local state refreshed.";
      this.syncSelections();
      this.renderAll();
    } catch (error) {
      this.status = error instanceof Error ? error.message : String(error);
      this.renderAll();
    }
  }

  syncSelections() {
    const profiles = this.data?.profiles || [];
    if (!profiles.some((profile) => profile.id === this.selectedProfileId)) {
      this.selectedProfileId = profiles[0]?.id || null;
    }

    const logs = this.data?.logs || [];
    if (!logs.some((log) => log.id === this.selectedLogId)) {
      this.selectedLogId = logs[0]?.id || null;
    }

    const models = this.data?.models || [];
    if (!models.some((model) => model.id === this.playground.model)) {
      this.playground.model = this.defaultPlaygroundModel() || "";
    }

    if (this.lastSecret && !this.ui.playgroundKeyInput.value) {
      this.ui.playgroundKeyInput.value = this.lastSecret;
      this.playground.key = this.lastSecret;
    }
  }

  renderAll() {
    this.rendering = true;
    this.renderHeader();
    this.renderSetupView();
    this.renderPlaygroundView();
    this.renderRequestsView();
    this.ui.statusLine.content = this.truncate(this.status, this.terminalWidth() - 2);
    this.ui.footer.content = this.footerHelp();
    this.rendering = false;
  }

  footerHelp() {
    if (this.compactTerminal()) {
      if (this.selectedTab === "setup") {
        return "Ctrl+S save · Ctrl+A auth · Ctrl+M sync · Ctrl+K key";
      }
      if (this.selectedTab === "playground") {
        return "Enter send · Ctrl+L clear · Ctrl+1/3 switch · Ctrl+R refresh";
      }
      return "Filter logs · Ctrl+1/2 switch · Ctrl+R refresh";
    }
    if (this.selectedTab === "setup") {
      return "Ctrl+S save  Ctrl+A auth  Ctrl+M sync  Ctrl+K key  Ctrl+O out  Ctrl+D del";
    }
    if (this.selectedTab === "playground") {
      return "Enter send  Ctrl+L clear  Ctrl+1 setup  Ctrl+3 reqs  Ctrl+R refresh";
    }
    return "Filter provider/status/query  Ctrl+1 setup  Ctrl+2 play  Ctrl+R refresh";
  }

  renderHeader() {
    const counts = this.data?.counts;
    const setup = this.data?.setup;
    const traffic = this.data?.traffic;
    this.ui.headerCounts.content = counts
      ? this.compactTerminal()
        ? `P ${counts.profiles} · M ${counts.models} · K ${counts.keys} · R ${counts.logs}`
        : `Providers ${counts.profiles} · Models ${counts.models} · Keys ${counts.keys} · Requests ${counts.logs}`
      : "Counts unavailable.";
    this.ui.headerNext.content = setup
      ? this.compactTerminal()
        ? this.truncate(`Next: ${setup.next_step} · ${traffic?.total_tokens ?? 0} tokens`, this.terminalWidth() - 4)
        : this.truncate(
            `Next: ${setup.next_step}  Latest traffic ${traffic?.total_tokens ?? 0} total tokens`,
            this.terminalWidth() - 4,
          )
      : "Waiting for operator state...";
  }

  renderSetupFormMeta() {
    const config = providerUiConfig(this.selectedProviderKind(), this.data?.providers || []);
    this.ui.providerNameInput.placeholder = `work-${config.suggestedName}`;
    this.ui.providerBaseUrlInput.placeholder = config.baseUrlPlaceholder;
    this.ui.providerBaseUrlInput.visible = config.supportsBaseUrl;
    this.ui.providerApiKeyInput.visible = config.requiresApiKey;
    this.ui.setupMeta.content = this.formatLines(
      this.compactTerminal()
        ? [
            config.helperTitle,
            config.helperBody,
            "Ctrl+S save · Ctrl+A auth",
            "Ctrl+M sync · Ctrl+K key",
          ]
        : [
            config.helperTitle,
            config.helperBody,
            "",
            "Ctrl+S save  Ctrl+A auth  Ctrl+M sync models  Ctrl+K create key  Ctrl+O logout  Ctrl+D delete provider",
          ],
      this.detailTextWidth(),
    );
  }

  renderSetupView() {
    const providers = this.data?.profiles || [];
    const providerListOptions = providers.length
      ? providers.map((profile) => ({
          name: this.truncate(`${profile.provider} · ${profile.name}`, this.listTextWidth()),
          description: this.truncate(`${profile.auth_label} · ${profile.model_count} models`, this.listTextWidth()),
          value: profile.id,
        }))
      : [{ name: "No providers yet", description: "Save the first provider on the right.", value: "empty" }];
    this.ui.providerList.options = providerListOptions;
    if (providerListOptions.length) {
      const selectedProfileIndex = providers.length
        ? Math.max(
            0,
            providers.findIndex((profile) => profile.id === this.selectedProfileId),
          )
        : 0;
      this.ui.providerList.setSelectedIndex(selectedProfileIndex === -1 ? 0 : selectedProfileIndex);
    }

    const providerOptions = (this.data?.providers || []).map((provider) => ({
      name: provider.kind,
      description: `${provider.class} · ${provider.auth_method === "browser_session" ? "browser auth" : "api key"}`,
      value: provider.kind,
    }));
    this.ui.providerKindSelect.options = providerOptions.length
      ? providerOptions
      : [{ name: "openai", description: "direct", value: "openai" }];
    const providerKindIndex = providerOptions.length
      ? Math.max(
          0,
          providerOptions.findIndex((provider) => provider.value === this.selectedProviderKind()),
        )
      : 0;
    this.ui.providerKindSelect.setSelectedIndex(providerKindIndex === -1 ? 0 : providerKindIndex);

    this.renderSetupFormMeta();

    const selectedProfile = providers.find((profile) => profile.id === this.selectedProfileId);
    const models = (this.data?.models || []).filter(
      (model) => model.provider === selectedProfile?.provider,
    );
    const keys = this.data?.keys || [];
    const selectedProviderDefinition = (this.data?.providers || []).find(
      (provider) => provider.kind === selectedProfile?.provider,
    );
    this.ui.providerDetails.content = selectedProfile
      ? this.formatLines(
          [
            `${selectedProfile.name} (${selectedProfile.provider})`,
            `${selectedProfile.auth_label} · ${selectedProfile.model_count} synced models`,
            selectedProfile.base_url ? `Base URL ${selectedProfile.base_url}` : "Base URL default",
            selectedProviderDefinition
              ? `Modes ${(selectedProviderDefinition.supports_chat_completions ? "chat/completions " : "")}${selectedProviderDefinition.supports_responses_api ? "responses" : ""}`.trim()
              : null,
            "",
            `Keys available ${keys.length}`,
            models.length ? `Ready model ${models[0].id}` : "Sync models to register provider/model ids.",
            "",
            this.compactTerminal()
              ? "Auth with Ctrl+A. Sync with Ctrl+M. Create one key with Ctrl+K."
              : "Use Ctrl+A to auth, Ctrl+M to sync, Ctrl+K to mint one Gunmetal key.",
            this.lastSecret ? `Latest key secret ${this.lastSecret}` : "Key secrets appear here when created.",
          ],
          this.listTextWidth(),
        )
      : this.formatLines(
          ["No provider selected.", "", "Save one provider, then auth it, sync models, and create a key."],
          this.listTextWidth(),
        );
  }

  renderPlaygroundView() {
    const models = this.data?.models || [];
    const modelOptions = models.length
      ? models.map((model) => ({
          name: model.id,
          description: model.provider,
          value: model.id,
        }))
      : [{ name: "No synced models", description: "Sync a provider first.", value: "empty" }];
    this.ui.playgroundModelSelect.options = modelOptions;
    this.ui.playgroundModelSelect.setSelectedIndex(
      models.length
        ? Math.max(
            0,
            models.findIndex((model) => model.id === this.playground.model),
          )
        : 0,
    );

    this.ui.playgroundModeSelect.setSelectedIndex(this.playground.mode === "responses" ? 1 : 0);
    this.ui.playgroundHistorySelect.setSelectedIndex(
      this.playground.historyMode === "single" ? 1 : 0,
    );
    this.ui.playgroundMeta.content = this.formatLines(
      [
        this.playground.model || "No model selected yet",
        this.playground.mode === "chat" ? "chat/completions" : "responses",
        this.playground.durationMs == null ? null : `${this.playground.durationMs} ms`,
        this.playground.usage?.total_tokens != null
          ? `tokens in ${this.playground.usage.input_tokens ?? 0} · out ${this.playground.usage.output_tokens ?? 0} · total ${this.playground.usage.total_tokens}`
          : "Usage lands in request history after each request.",
        !this.ui.playgroundKeyInput.value.trim()
          ? "Create a key in Setup with Ctrl+K or paste one here."
          : this.playground.running
            ? "Streaming response..."
            : "Enter sends. Ctrl+L clears.",
      ],
      this.detailTextWidth(),
    );

    this.clearChildren(this.ui.playgroundTranscript);
    if (!this.playground.messages.length) {
      this.ui.playgroundTranscript.add(
        new TextRenderable(this.renderer, {
          content: "No playground traffic yet. Paste one Gunmetal key, pick a synced model, and send the first message.",
          textColor: "#9aa3ad",
        }),
      );
      return;
    }

    for (const message of this.playground.messages) {
      const card = new BoxRenderable(this.renderer, {
        width: "100%",
        border: true,
        borderStyle: "rounded",
        borderColor: message.role === "assistant" ? "#7dd3c8" : "#d8b26e",
        padding: 1,
        marginBottom: 1,
        backgroundColor: "#0d1013",
        flexDirection: "column",
      });
      card.add(
        new TextRenderable(this.renderer, {
          content: message.role,
          textColor: "#9aa3ad",
        }),
      );
      card.add(
        new TextRenderable(this.renderer, {
          content: message.content || "...",
          textColor: "#f4f4ee",
        }),
      );
      this.ui.playgroundTranscript.add(card);
    }
    this.ui.playgroundTranscript.scrollTo(this.ui.playgroundTranscript.scrollHeight);
  }

  renderRequestsView() {
    this.ui.requestQueryInput.value = this.requestFilters.query;
    const logs = (this.data?.logs || []).filter((log) => this.logMatchesRequestFilters(log));
    const providerOptions = [
      { name: "all providers", value: "all" },
      ...((this.data?.provider_summaries || []).map((item) => ({
        name: `${item.label} · ${item.provider}`,
        value: this.providerFilterValue(item.provider, item.profile_name),
      }))),
    ];
    this.ui.requestProviderFilter.options = providerOptions;
    this.ui.requestProviderFilter.setSelectedIndex(
      Math.max(
        0,
        providerOptions.findIndex((item) => item.value === this.requestFilters.provider),
      ),
    );

    this.ui.requestStatusFilter.setSelectedIndex(
      this.requestFilters.status === "success" ? 1 : this.requestFilters.status === "error" ? 2 : 0,
    );

    const requestOptions = logs.length
      ? logs.map((log) => ({
          name: this.truncate(`${log.profile_name || log.provider} · ${log.status_code ?? "pending"}`, this.listTextWidth()),
          description: this.truncate(`${log.model} · ${log.request_mode || "request"} · ${log.duration_ms}ms`, this.listTextWidth()),
          value: log.id,
        }))
      : [{ name: "No requests yet", description: "Use the playground or any compatible app.", value: "empty" }];
    this.ui.requestList.options = requestOptions;
    this.ui.requestList.setSelectedIndex(
      logs.length
        ? Math.max(
            0,
            logs.findIndex((log) => log.id === this.selectedLogId),
          )
        : 0,
    );

    const selectedLog = logs.find((log) => log.id === this.selectedLogId) || logs[0];
    if (selectedLog) {
      this.selectedLogId = selectedLog.id;
    }
    const traffic = this.data?.traffic;
    this.ui.requestTraffic.content = traffic
      ? this.formatLines(
          [
            this.compactTerminal()
              ? `Recent ${traffic.recent_requests} · Ok ${traffic.success_count} · Err ${traffic.error_count}`
              : `Recent ${traffic.recent_requests} · Success ${traffic.success_count} · Errors ${traffic.error_count}`,
            `Total tokens ${traffic.total_tokens} · Avg latency ${traffic.avg_latency_ms ?? "—"} ms`,
          ],
          this.detailTextWidth(),
        )
      : "Traffic summary unavailable.";
    const providerSummary = (this.data?.provider_summaries || [])
      .slice(0, 3)
      .map(
        (item) =>
          `${item.label} · ${item.provider} · ${item.requests} req · ${item.total_tokens} tok · ${item.error_count} err`,
      );
    const modelSummary = (this.data?.model_summaries || [])
      .slice(0, 3)
      .map(
        (item) =>
          `${item.model} · ${item.requests} req · ${item.total_tokens} tok · ${item.avg_latency_ms ?? "—"} ms`,
      );
    this.ui.requestSummary.content = this.formatLines(
      [
        "Top providers",
        ...(providerSummary.length ? providerSummary : ["No provider traffic yet."]),
        "",
        "Top models",
        ...(modelSummary.length ? modelSummary : ["No model traffic yet."]),
      ],
      this.detailTextWidth(),
    );
    this.ui.requestDetails.content = selectedLog
      ? this.formatLines(
          [
            `${selectedLog.model}`,
            `via ${selectedLog.profile_name || selectedLog.provider}`,
            `status ${selectedLog.status_code ?? "pending"} · ${selectedLog.duration_ms} ms`,
            `tokens in ${selectedLog.input_tokens ?? 0} · out ${selectedLog.output_tokens ?? 0} · total ${selectedLog.total_tokens ?? 0}`,
            `key ${selectedLog.key_name || "unknown"} · mode ${selectedLog.request_mode || "request"}`,
            `${selectedLog.endpoint} · ${selectedLog.started_at}`,
            selectedLog.error_message ? `error ${selectedLog.error_message}` : "request completed without recorded provider error",
          ],
          this.detailTextWidth(),
        )
      : logs.length
        ? "Select one request from the filtered list."
        : "Use the playground or any OpenAI-compatible app and the first request will appear here.";
  }

  providerFilterValue(provider, profileName) {
    return `${provider}::${profileName || ""}`;
  }

  logMatchesRequestFilters(log) {
    const providerMatches =
      this.requestFilters.provider === "all" ||
      this.providerFilterValue(log.provider, log.profile_name) === this.requestFilters.provider;
    const statusMatches =
      this.requestFilters.status === "all" ||
      (this.requestFilters.status === "success"
        ? (log.status_code ?? 0) < 400 && !log.error_message
        : (log.status_code ?? 0) >= 400 || Boolean(log.error_message));
    const query = this.requestFilters.query.trim().toLowerCase();
    const queryMatches =
      !query ||
      [
        log.provider,
        log.profile_name || "",
        log.model,
        log.endpoint,
        log.key_name || "",
        log.request_mode || "",
        log.error_message || "",
      ]
        .join(" ")
        .toLowerCase()
        .includes(query);
    return providerMatches && statusMatches && queryMatches;
  }

  selectedProviderKind() {
    const option = this.ui.providerKindSelect?.getSelectedOption();
    return String(option?.value || "openai");
  }

  selectedProfile() {
    return (this.data?.profiles || []).find((profile) => profile.id === this.selectedProfileId) || null;
  }

  modelsForProfile(profile) {
    if (!profile) {
      return [];
    }
    return (this.data?.models || []).filter((model) => model.provider === profile.provider);
  }

  defaultPlaygroundModel() {
    const selected = this.selectedProfile();
    const profileModel = this.modelsForProfile(selected)[0]?.id;
    if (profileModel) {
      return profileModel;
    }
    return (this.data?.models || [])[0]?.id || "";
  }

  async saveProvider() {
    const provider = this.selectedProviderKind();
    const payload = {
      provider,
      name: this.ui.providerNameInput.value.trim(),
      base_url: this.ui.providerBaseUrlInput.visible
        ? this.ui.providerBaseUrlInput.value.trim()
        : "",
      api_key: this.ui.providerApiKeyInput.visible
        ? this.ui.providerApiKeyInput.value.trim()
        : "",
    };
    if (!payload.name) {
      this.status = "Provider name is required.";
      this.renderAll();
      return;
    }

    const result = await submitAction("/app/api/profiles", payload);
    this.status = result.message;
    this.ui.providerNameInput.value = "";
    this.ui.providerBaseUrlInput.value = "";
    this.ui.providerApiKeyInput.value = "";
    await this.refreshState();
  }

  async providerAction(action) {
    const profile = this.selectedProfile();
    if (!profile) {
      this.status = "Select a provider first.";
      this.renderAll();
      return;
    }
    const result = await submitAction(`/app/api/profiles/${profile.id}/${action}`, {});
    this.status = result.message;
    await this.refreshState();
    if (action === "sync") {
      this.playground.model = this.defaultPlaygroundModel();
      this.renderAll();
    }
  }

  async createKey() {
    const profile = this.selectedProfile();
    if (!profile) {
      this.status = "Select a provider first.";
      this.renderAll();
      return;
    }

    const result = await submitAction(`/app/api/profiles/${profile.id}/keys`, {
      name: `${profile.name}-key`,
    });
    this.status = result.message;
    this.lastSecret = result.secret || this.lastSecret;
    if (result.secret) {
      this.ui.playgroundKeyInput.value = result.secret;
      this.playground.key = result.secret;
    }
    await this.refreshState();
    this.playground.model = this.defaultPlaygroundModel();
    this.status = result.secret
      ? "Key created. Playground is ready for the first request."
      : this.status;
    this.selectTab(1);
  }

  async deleteProfile() {
    const profile = this.selectedProfile();
    if (!profile) {
      this.status = "Select a provider first.";
      this.renderAll();
      return;
    }

    const response = await fetch(`${baseUrl}/app/api/profiles/${profile.id}`, {
      method: "DELETE",
    });
    const body = await parseResponse(response);
    this.status = body.message;
    await this.refreshState();
  }

  async sendPlayground() {
    const key = this.ui.playgroundKeyInput.value.trim();
    const prompt = this.ui.playgroundInput.value.trim();
    const model = this.playground.model;
    if (!key) {
      this.status = "Paste a Gunmetal key first.";
      this.renderAll();
      return;
    }
    if (!model) {
      this.status = "Sync at least one model first.";
      this.renderAll();
      return;
    }
    if (!prompt) {
      return;
    }

    if (this.playground.historyMode === "single") {
      this.playground.messages = [];
    }
    this.playground.key = key;
    this.playground.messages.push({ role: "user", content: prompt });
    this.ui.playgroundInput.value = "";
    this.playground.running = true;
    this.playground.usage = null;
    this.playground.durationMs = null;
    this.renderAll();

    try {
      const result = await runPlaygroundRequest(
        key,
        model,
        this.playground.mode,
        this.playground.messages,
        (delta) => {
          const current = this.playground.messages[this.playground.messages.length - 1];
          if (!current || current.role !== "assistant") {
            this.playground.messages.push({ role: "assistant", content: delta });
          } else {
            current.content = delta;
          }
          this.renderPlaygroundView();
        },
      );
      this.playground.usage = result.usage;
      this.playground.durationMs = result.durationMs;
      if (
        !this.playground.messages.length ||
        this.playground.messages[this.playground.messages.length - 1].role !== "assistant"
      ) {
        this.playground.messages.push({ role: "assistant", content: result.content });
      } else {
        this.playground.messages[this.playground.messages.length - 1].content = result.content;
      }
      this.status = "Playground request completed.";
      this.selectedTab = "requests";
      this.ui.tabs.setSelectedIndex(2);
      this.syncVisibleView();
      await this.refreshState();
    } catch (error) {
      this.playground.messages = this.playground.messages.slice(
        0,
        Math.max(0, this.playground.messages.length - 1),
      );
      this.status = error instanceof Error ? error.message : String(error);
      this.renderAll();
    } finally {
      this.playground.running = false;
      this.renderAll();
    }
  }

  clearChildren(renderable) {
    for (const child of renderable.getChildren()) {
      renderable.remove(child.id);
    }
  }

  terminalWidth() {
    return Math.max(60, process.stdout?.columns || 100);
  }

  compactTerminal() {
    return this.terminalWidth() <= 90;
  }

  listTextWidth() {
    return this.compactTerminal() ? 24 : 40;
  }

  detailTextWidth() {
    return this.compactTerminal() ? 44 : 72;
  }

  truncate(text, width) {
    const value = String(text ?? "");
    if (value.length <= width) {
      return value;
    }
    if (width <= 3) {
      return value.slice(0, width);
    }
    return `${value.slice(0, width - 3)}...`;
  }

  formatLines(lines, width) {
    return lines
      .filter((line) => line !== null && line !== undefined)
      .map((line) => {
        const value = String(line);
        if (!value) {
          return "";
        }
        return this.truncate(value, width);
      })
      .join("\n");
  }
}

async function fetchJson(path, init) {
  const response = await fetch(`${baseUrl}${path}`, init);
  return parseResponse(response);
}

async function submitAction(path, body) {
  return fetchJson(path, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
}

async function parseResponse(response) {
  const text = await response.text();
  const body = text ? JSON.parse(text) : {};
  if (!response.ok) {
    throw new Error(body?.error?.message || body?.message || "request failed");
  }
  return body;
}

function providerUiConfig(kind, providers) {
  const provider = providers.find((item) => item.kind === kind);
  if (provider) {
    const requestModes =
      [provider.supports_chat_completions ? "chat/completions" : null, provider.supports_responses_api ? "responses" : null]
        .filter(Boolean)
        .join(" + ") || "request";
    return {
      helperTitle: provider.helper_title || provider.label || provider.kind || "Provider",
      helperBody: `${provider.helper_body || "Save one provider first, then auth it, sync models, and create one key."} Request modes: ${requestModes}.`,
      requiresApiKey: provider.auth_method === "api_key",
      supportsBaseUrl: Boolean(provider.supports_base_url),
      baseUrlPlaceholder: provider.base_url_placeholder || "optional base URL",
      suggestedName: provider.suggested_name || provider.kind || "provider",
    };
  }

  return {
    helperTitle: "Provider",
    helperBody: "Save one provider first, then auth it, sync models, and create one key.",
    requiresApiKey: true,
    supportsBaseUrl: true,
    baseUrlPlaceholder: "optional override",
    suggestedName: kind || "provider",
  };
}

function playgroundPayload(mode, model, messages) {
  if (mode === "responses") {
    return {
      model,
      stream: true,
      input: messages.map((message) => ({
        role: message.role === "system" ? "developer" : message.role,
        content: [{ type: "input_text", text: message.content }],
      })),
    };
  }

  return {
    model,
    stream: true,
    messages,
  };
}

async function runPlaygroundRequest(key, model, mode, messages, onDelta) {
  const startedAt = performance.now();
  const response = await fetch(`${baseUrl}${mode === "responses" ? "/v1/responses" : "/v1/chat/completions"}`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${key}`,
    },
    body: JSON.stringify(playgroundPayload(mode, model, messages)),
  });

  if (!response.ok) {
    const text = await response.text();
    try {
      const body = text ? JSON.parse(text) : {};
      throw new Error(body?.error?.message || body?.message || "request failed");
    } catch (error) {
      throw new Error(error instanceof Error ? error.message : text || "request failed");
    }
  }

  let content = "";
  let usage = null;
  await readSse(response, ({ event, data }) => {
    if (data === "[DONE]") {
      return;
    }
    const parsed = JSON.parse(data);
    if (mode === "chat") {
      const delta = parsed?.choices?.[0]?.delta?.content || "";
      if (delta) {
        content += delta;
        onDelta(content);
      }
      return;
    }

    if (event === "response.output_text.delta") {
      content += parsed.delta || "";
      onDelta(content);
    } else if (event === "response.completed") {
      content = parsed?.response?.output_text || content;
      usage = parsed?.response?.usage || null;
      onDelta(content);
    }
  });

  return {
    content,
    usage,
    durationMs: Math.round(performance.now() - startedAt),
  };
}

async function readSse(response, onEvent) {
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  while (true) {
    const { value, done } = await reader.read();
    buffer += decoder.decode(value || new Uint8Array(), { stream: !done });
    let boundary = buffer.indexOf("\n\n");
    while (boundary !== -1) {
      const rawEvent = buffer.slice(0, boundary);
      buffer = buffer.slice(boundary + 2);
      const event = parseSse(rawEvent);
      if (event) {
        onEvent(event);
      }
      boundary = buffer.indexOf("\n\n");
    }
    if (done) {
      break;
    }
  }
}

function parseSse(rawEvent) {
  let event = "message";
  const data = [];
  for (const line of rawEvent.split(/\r?\n/)) {
    if (line.startsWith("event:")) {
      event = line.slice(6).trim();
    } else if (line.startsWith("data:")) {
      data.push(line.slice(5).trimStart());
    }
  }
  return data.length ? { event, data: data.join("\n") } : null;
}

await new GunmetalTuiApp().start();
