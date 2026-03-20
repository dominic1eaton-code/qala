import { Component } from '@angular/core';
import { NgFor } from '@angular/common';
import { RouterLink } from '@angular/router';

interface InviteRow {
  email: string;
  role: string;
}

@Component({
  selector: 'app-onboarding-invite',
  standalone: true,
  imports: [NgFor, RouterLink],
  templateUrl: './onboarding-invite.component.html'
})
export class OnboardingInviteComponent {
  invites: InviteRow[] = [
    { email: '', role: 'Developer' },
    { email: '', role: 'QA / Test' }
  ];

  addRow(): void {
    this.invites.push({ email: '', role: 'Developer' });
  }

  removeRow(index: number): void {
    if (this.invites.length > 1) {
      this.invites.splice(index, 1);
    }
  }
}
