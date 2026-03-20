import { Component } from '@angular/core';
import { NgFor } from '@angular/common';
import { RouterLink } from '@angular/router';

interface SdeTemplate {
  id: string;
  name: string;
  description: string;
}

@Component({
  selector: 'app-onboarding-sde',
  standalone: true,
  imports: [NgFor, RouterLink],
  templateUrl: './onboarding-sde.component.html'
})
export class OnboardingSdeComponent {
  templates: SdeTemplate[] = [
    { id: 'frontend', name: 'Frontend Dev SDE', description: 'Node, TypeScript, Web build toolchain.' },
    { id: 'backend', name: 'Backend Dev SDE', description: 'Go, CI pipelines, container build stack.' },
    { id: 'data', name: 'Data Platform SDE', description: 'Scala, Spark, analytics tools.' },
    { id: 'security', name: 'Security SEM SDE', description: 'SAST/SCA tooling with alerting.' }
  ];
  selectedTemplate = 'frontend';

  selectTemplate(id: string): void {
    this.selectedTemplate = id;
  }
}
