import { Component } from '@angular/core';
import { NgFor } from '@angular/common';
import { RouterLink } from '@angular/router';

interface FactoryTier {
  id: string;
  name: string;
  description: string;
}

@Component({
  selector: 'app-onboarding-factory',
  standalone: true,
  imports: [NgFor, RouterLink],
  templateUrl: './onboarding-factory.component.html',
  styleUrl: './auth.styles.css'
})
export class OnboardingFactoryComponent {
  namespace = '';
  tiers: FactoryTier[] = [
    { id: 'personal', name: 'Personal', description: 'Solo developer. Lightweight governance.' },
    { id: 'team', name: 'Team', description: 'Small team. Shared pipelines & templates.' },
    { id: 'domain', name: 'Domain', description: 'Business unit or product domain.' },
    { id: 'enterprise', name: 'Enterprise', description: 'Multi-team. Full governance suite.' }
  ];
  selectedTier = 'personal';

  selectTier(id: string): void {
    this.selectedTier = id;
  }

  formatNamespace(value: string): void {
    this.namespace = value.toLowerCase().replace(/[^a-z0-9-]/g, '-').replace(/--+/g, '-');
  }
}


