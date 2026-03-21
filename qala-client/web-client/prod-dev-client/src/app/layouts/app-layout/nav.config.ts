export interface NavItem {
  label: string;
  route: string;
}

export interface NavSystem {
  label: string;
  icon: string;
  route: string;
  children: NavItem[];
}

export const NAV_SYSTEMS: NavSystem[] = [
  {
    label: 'Overview',
    icon: 'O',
    route: '/overview',
    children: [
      { label: 'Factory Dashboard', route: '/overview/dashboard' },
      { label: 'App Dashboard', route: '/overview/app-dashboard' },
      { label: 'SFOS Dashboard', route: '/overview/legacy-dashboard' }
    ]
  },
  {
    label: 'Portfolio',
    icon: 'P',
    route: '/portfolio',
    children: [
      { label: 'Portfolio', route: '/portfolio/portfolio' },
      { label: 'Value Chain', route: '/portfolio/value-chain' },
      { label: 'Roadmaps', route: '/portfolio/roadmaps' },
      { label: 'Timelines & Gantt', route: '/portfolio/timelines' },
      { label: 'Multi-Solution Tracking', route: '/portfolio/multi-solution' }
    ]
  },
  {
    label: 'Solutions',
    icon: 'S',
    route: '/solutions',
    children: [
      { label: 'Solutions', route: '/solutions/solutions' },
      { label: 'Solution Workbench', route: '/solutions/workbench' },
      { label: 'Solution Registry', route: '/solutions/registry' },
      { label: 'Solution Detail', route: '/solutions/detail' },
      { label: 'Solution Book', route: '/solutions/book' },
      { label: 'Solution Boards', route: '/solutions/boards' },
      { label: 'Solution Channels', route: '/solutions/channels' },
      { label: 'App Solutions', route: '/solutions/app-solutions' },
      { label: 'Legacy Registry (SFOS)', route: '/solutions/legacy-registry' }
    ]
  },
  {
    label: 'Factory',
    icon: 'F',
    route: '/factory',
    children: [
      { label: 'Solution Factory', route: '/factory/solution-factory' },
      { label: 'Environments', route: '/factory/environments' },
      { label: 'Workspaces', route: '/factory/workspaces' },
      { label: 'SDEs', route: '/factory/sdes' },
      { label: 'Tools', route: '/factory/tools' },
      { label: 'App Factories', route: '/factory/app-factories' },
      { label: 'Legacy Factory (SFOS)', route: '/factory/legacy-solution-factory' },
      { label: 'Legacy SDE Manager', route: '/factory/legacy-sde-manager' },
      { label: 'Legacy Workspace', route: '/factory/legacy-workspace' },
      { label: 'Legacy SDE Tooling', route: '/factory/legacy-sde-tooling' }
    ]
  },
  {
    label: 'Pipelines',
    icon: 'C',
    route: '/pipelines',
    children: [
      { label: 'Pipelines', route: '/pipelines/pipelines' },
      { label: 'Version Control & CM', route: '/pipelines/version-control' },
      { label: 'Releases', route: '/pipelines/releases' },
      { label: 'Release Schedule', route: '/pipelines/release-schedule' },
      { label: 'Solution Schedules', route: '/pipelines/solution-schedules' },
      { label: 'Legacy CI/CD (SFOS)', route: '/pipelines/legacy-cicd' },
      { label: 'App Pipelines', route: '/pipelines/app-pipelines' }
    ]
  },
  {
    label: 'Artifacts',
    icon: 'A',
    route: '/artifacts',
    children: [
      { label: 'Artifacts', route: '/artifacts/artifacts' },
      { label: 'Warehouse', route: '/artifacts/warehouse' },
      { label: 'Solution Packages', route: '/artifacts/packages' },
      { label: 'Legacy Artifacts (SFOS)', route: '/artifacts/legacy-artifacts' }
    ]
  },
  {
    label: 'Intelligence',
    icon: 'I',
    route: '/intelligence',
    children: [
      { label: 'AI Insights', route: '/intelligence/ai-insights' },
      { label: 'Data Platform', route: '/intelligence/data-platform' },
      { label: 'Notifications', route: '/intelligence/notifications' }
    ]
  },
  {
    label: 'Governance',
    icon: 'G',
    route: '/governance',
    children: [
      { label: 'Security', route: '/governance/security' },
      { label: 'Governance', route: '/governance/governance' },
      { label: 'Playbook', route: '/governance/playbook' },
      { label: 'Settings', route: '/governance/settings' }
    ]
  }
];
