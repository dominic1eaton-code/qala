import { Routes } from '@angular/router';
import { AuthLayoutComponent } from './layouts/auth-layout/auth-layout.component';
import { AppLayoutComponent } from './layouts/app-layout/app-layout.component';
import { HomeLayoutComponent } from './layouts/home-layout/home-layout.component';
import { LoginComponent } from './pages/auth/login.component';
import { RegistrationComponent } from './pages/auth/registration.component';
import { OnboardingComponent } from './pages/auth/onboarding.component';
import { OnboardingFactoryComponent } from './pages/auth/onboarding-factory.component';
import { OnboardingInviteComponent } from './pages/auth/onboarding-invite.component';
import { OnboardingSdeComponent } from './pages/auth/onboarding-sde.component';
import { LoadingComponent } from './pages/auth/loading.component';
import { OverviewDashboardComponent } from './pages/overview/dashboard.component';
import { OverviewAppDashboardComponent } from './pages/overview/app-dashboard.component';
import { OverviewLegacyDashboardComponent } from './pages/overview/legacy-dashboard.component';
import { PortfolioOverviewComponent } from './pages/portfolio/portfolio.component';
import { PortfolioValueChainComponent } from './pages/portfolio/value-chain.component';
import { PortfolioRoadmapsComponent } from './pages/portfolio/roadmaps.component';
import { PortfolioTimelinesComponent } from './pages/portfolio/timelines.component';
import { PortfolioMultiSolutionComponent } from './pages/portfolio/multi-solution.component';
import { SolutionsListComponent } from './pages/solutions/solutions.component';
import { SolutionsRegistryComponent } from './pages/solutions/registry.component';
import { SolutionsDetailComponent } from './pages/solutions/detail.component';
import { SolutionsBookComponent } from './pages/solutions/book.component';
import { SolutionsBoardsComponent } from './pages/solutions/boards.component';
import { SolutionsChannelsComponent } from './pages/solutions/channels.component';
import { SolutionsLegacyRegistryComponent } from './pages/solutions/legacy-registry.component';
import { SolutionsAppComponent } from './pages/solutions/app-solutions.component';
import { FactorySolutionFactoryComponent } from './pages/factory/solution-factory.component';
import { FactoryEnvironmentsComponent } from './pages/factory/environments.component';
import { FactoryWorkspacesComponent } from './pages/factory/workspaces.component';
import { FactorySdesComponent } from './pages/factory/sdes.component';
import { FactoryToolsComponent } from './pages/factory/tools.component';
import { FactoryAppFactoriesComponent } from './pages/factory/app-factories.component';
import { FactoryLegacySolutionFactoryComponent } from './pages/factory/legacy-solution-factory.component';
import { FactoryLegacySdeManagerComponent } from './pages/factory/legacy-sde-manager.component';
import { FactoryLegacyWorkspaceComponent } from './pages/factory/legacy-workspace.component';
import { FactoryLegacySdeToolingComponent } from './pages/factory/legacy-sde-tooling.component';
import { PipelinesOverviewComponent } from './pages/pipelines/pipelines.component';
import { PipelinesVersionControlComponent } from './pages/pipelines/version-control.component';
import { PipelinesReleasesComponent } from './pages/pipelines/releases.component';
import { PipelinesReleaseScheduleComponent } from './pages/pipelines/release-schedule.component';
import { PipelinesSolutionSchedulesComponent } from './pages/pipelines/solution-schedules.component';
import { PipelinesLegacyCicdComponent } from './pages/pipelines/legacy-cicd.component';
import { PipelinesAppComponent } from './pages/pipelines/app-pipelines.component';
import { ArtifactsOverviewComponent } from './pages/artifacts/artifacts.component';
import { ArtifactsWarehouseComponent } from './pages/artifacts/warehouse.component';
import { ArtifactsPackagesComponent } from './pages/artifacts/packages.component';
import { ArtifactsLegacyComponent } from './pages/artifacts/legacy-artifacts.component';
import { IntelligenceAiInsightsComponent } from './pages/intelligence/ai-insights.component';
import { IntelligenceDataPlatformComponent } from './pages/intelligence/data-platform.component';
import { IntelligenceNotificationsComponent } from './pages/intelligence/notifications.component';
import { GovernanceSecurityComponent } from './pages/governance/security.component';
import { GovernanceOverviewComponent } from './pages/governance/governance.component';
import { GovernancePlaybookComponent } from './pages/governance/playbook.component';
import { GovernanceSettingsComponent } from './pages/governance/settings.component';
import { HomeComponent } from './pages/home/home.component';
import { HomeFactoryComponent } from './pages/home/home-factory.component';
import { HomeLifecycleComponent } from './pages/home/home-lifecycle.component';
import { HomeSolutionComponent } from './pages/home/home-solution.component';

export const routes: Routes = [
  { path: '', redirectTo: 'home', pathMatch: 'full' },
    {
      path: 'home',
      component: HomeLayoutComponent,
      children: [
        { path: '', component: HomeComponent },
        { path: 'solution', component: HomeSolutionComponent },
        { path: 'factory', component: HomeFactoryComponent },
        { path: 'lifecycle', component: HomeLifecycleComponent }
      ]
    },
    {
      path: '',
      component: AuthLayoutComponent,
      children: [
        { path: 'login', component: LoginComponent },
        { path: 'register', component: RegistrationComponent },
        { path: 'onboarding', component: OnboardingComponent },
        { path: 'onboarding/factory', component: OnboardingFactoryComponent },
        { path: 'onboarding/invite', component: OnboardingInviteComponent },
        { path: 'onboarding/sde', component: OnboardingSdeComponent },
        { path: 'loading', component: LoadingComponent }
      ]
    },
    {
      path: '',
      component: AppLayoutComponent,
      children: [
    {
      path: 'overview',
      children: [
      { path: 'dashboard', component: OverviewDashboardComponent },
      { path: 'app-dashboard', component: OverviewAppDashboardComponent },
      { path: 'legacy-dashboard', component: OverviewLegacyDashboardComponent },
      { path: '', redirectTo: 'dashboard', pathMatch: 'full' },
      ]
    },
    {
      path: 'portfolio',
      children: [
      { path: 'portfolio', component: PortfolioOverviewComponent },
      { path: 'value-chain', component: PortfolioValueChainComponent },
      { path: 'roadmaps', component: PortfolioRoadmapsComponent },
      { path: 'timelines', component: PortfolioTimelinesComponent },
      { path: 'multi-solution', component: PortfolioMultiSolutionComponent },
      { path: '', redirectTo: 'portfolio', pathMatch: 'full' },
      ]
    },
    {
      path: 'solutions',
      children: [
      { path: 'solutions', component: SolutionsListComponent },
      { path: 'registry', component: SolutionsRegistryComponent },
      { path: 'detail', component: SolutionsDetailComponent },
      { path: 'book', component: SolutionsBookComponent },
      { path: 'boards', component: SolutionsBoardsComponent },
      { path: 'channels', component: SolutionsChannelsComponent },
      { path: 'legacy-registry', component: SolutionsLegacyRegistryComponent },
      { path: 'app-solutions', component: SolutionsAppComponent },
      { path: '', redirectTo: 'solutions', pathMatch: 'full' },
      ]
    },
    {
      path: 'factory',
      children: [
      { path: 'solution-factory', component: FactorySolutionFactoryComponent },
      { path: 'environments', component: FactoryEnvironmentsComponent },
      { path: 'workspaces', component: FactoryWorkspacesComponent },
      { path: 'sdes', component: FactorySdesComponent },
      { path: 'tools', component: FactoryToolsComponent },
      { path: 'app-factories', component: FactoryAppFactoriesComponent },
      { path: 'legacy-solution-factory', component: FactoryLegacySolutionFactoryComponent },
      { path: 'legacy-sde-manager', component: FactoryLegacySdeManagerComponent },
      { path: 'legacy-workspace', component: FactoryLegacyWorkspaceComponent },
      { path: 'legacy-sde-tooling', component: FactoryLegacySdeToolingComponent },
      { path: '', redirectTo: 'solution-factory', pathMatch: 'full' },
      ]
    },
    {
      path: 'pipelines',
      children: [
      { path: 'pipelines', component: PipelinesOverviewComponent },
      { path: 'version-control', component: PipelinesVersionControlComponent },
      { path: 'releases', component: PipelinesReleasesComponent },
      { path: 'release-schedule', component: PipelinesReleaseScheduleComponent },
      { path: 'solution-schedules', component: PipelinesSolutionSchedulesComponent },
      { path: 'legacy-cicd', component: PipelinesLegacyCicdComponent },
      { path: 'app-pipelines', component: PipelinesAppComponent },
      { path: '', redirectTo: 'pipelines', pathMatch: 'full' },
      ]
    },
    {
      path: 'artifacts',
      children: [
      { path: 'artifacts', component: ArtifactsOverviewComponent },
      { path: 'warehouse', component: ArtifactsWarehouseComponent },
      { path: 'packages', component: ArtifactsPackagesComponent },
      { path: 'legacy-artifacts', component: ArtifactsLegacyComponent },
      { path: '', redirectTo: 'artifacts', pathMatch: 'full' },
      ]
    },
    {
      path: 'intelligence',
      children: [
      { path: 'ai-insights', component: IntelligenceAiInsightsComponent },
      { path: 'data-platform', component: IntelligenceDataPlatformComponent },
      { path: 'notifications', component: IntelligenceNotificationsComponent },
      { path: '', redirectTo: 'ai-insights', pathMatch: 'full' },
      ]
    },
    {
      path: 'governance',
      children: [
      { path: 'security', component: GovernanceSecurityComponent },
      { path: 'governance', component: GovernanceOverviewComponent },
      { path: 'playbook', component: GovernancePlaybookComponent },
      { path: 'settings', component: GovernanceSettingsComponent },
      { path: '', redirectTo: 'security', pathMatch: 'full' },
      ]
    },
        { path: '', redirectTo: 'overview/dashboard', pathMatch: 'full' }
      ]
    },
  { path: '**', redirectTo: 'home' }
];
