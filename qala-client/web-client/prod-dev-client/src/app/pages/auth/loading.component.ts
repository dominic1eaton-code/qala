import { Component, OnDestroy, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { NgFor, NgClass, NgIf, DecimalPipe } from '@angular/common';

type StepStatus = 'done' | 'active' | 'pending';

interface ProgressStep {
  name: string;
  detail: string;
  status: StepStatus;
}

interface AdvanceStep {
  stepIndex: number;
  pct: number;
  detail: string;
  status: 'RUNNING' | 'DONE';
}

@Component({
  selector: 'app-loading',
  standalone: true,
  imports: [NgFor, NgClass, NgIf, DecimalPipe],
  templateUrl: './loading.component.html',
  styleUrl: './auth.styles.css'
})
export class LoadingComponent implements OnInit, OnDestroy {
  progress = 25;
  percentLabel = '25%';
  timers: number[] = [];

  steps: ProgressStep[] = [
    {
      name: 'Account Created',
      detail: 'user-identity-service Â· user record committed',
      status: 'done'
    },
    {
      name: 'Factory Record Registered',
      detail: 'sde-management-service Â· factory-id: fab_9x2k',
      status: 'done'
    },
    {
      name: 'Provisioning SDE',
      detail: 'applying Go Backend template',
      status: 'active'
    },
    {
      name: 'Installing Toolchain',
      detail: 'go Â· docker Â· terraform Â· golint Â· gotest',
      status: 'pending'
    },
    {
      name: 'Configuring CI Pipeline',
      detail: 'workflow-ci-cd-service Â· registering triggers',
      status: 'pending'
    },
    {
      name: 'Registering Solution',
      detail: 'solution-registry Â· type: Application Â· stage: SANDBOX',
      status: 'pending'
    },
    {
      name: 'Applying Security Baseline',
      detail: 'security-sem-service Â· SAST Â· SCA Â· RBAC policies',
      status: 'pending'
    },
    {
      name: 'Factory Ready',
      detail: 'redirecting to workspaceâ€¦',
      status: 'pending'
    }
  ];

  sequence: AdvanceStep[] = [
    { stepIndex: 3, pct: 38, status: 'RUNNING', detail: 'resolving dependency lock fileâ€¦' },
    { stepIndex: 3, pct: 50, status: 'RUNNING', detail: 'installing go 1.22.0 Â· docker 24.0' },
    { stepIndex: 4, pct: 62, status: 'RUNNING', detail: 'configuring build triggersâ€¦' },
    { stepIndex: 4, pct: 72, status: 'RUNNING', detail: 'pipeline stages registered' },
    { stepIndex: 5, pct: 80, status: 'RUNNING', detail: 'solution record: sol_8m3n' },
    { stepIndex: 6, pct: 88, status: 'RUNNING', detail: 'scanning baseline policiesâ€¦' },
    { stepIndex: 6, pct: 94, status: 'RUNNING', detail: 'RBAC policies applied' },
    { stepIndex: 7, pct: 100, status: 'DONE', detail: 'all systems operational' }
  ];

  constructor(private router: Router) {}

  ngOnInit(): void {
    this.scheduleAdvance(1800);
  }

  ngOnDestroy(): void {
    this.timers.forEach((t) => window.clearTimeout(t));
  }

  private scheduleAdvance(delay: number): void {
    const timer = window.setTimeout(() => {
      this.advance();
    }, delay);
    this.timers.push(timer);
  }

  private advance(): void {
    const next = this.sequence.shift();
    if (!next) {
      const timer = window.setTimeout(() => {
        this.router.navigateByUrl('/overview/dashboard');
      }, 1200);
      this.timers.push(timer);
      return;
    }

    const prevIndex = this.steps.findIndex((s) => s.status === 'active');
    if (prevIndex >= 0) {
      this.steps[prevIndex].status = 'done';
    }

    this.steps.forEach((step, idx) => {
      if (idx === next.stepIndex) {
        step.status = 'active';
        step.detail = next.detail;
      }
    });

    this.progress = next.pct;
    this.percentLabel = `${next.pct}%`;

    const delay = this.sequence.length === 0 ? 800 : 1600;
    this.scheduleAdvance(delay);
  }
}


