import { Component } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { NgClass } from '@angular/common';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-registration',
  standalone: true,
  imports: [FormsModule, NgClass, RouterLink],
  templateUrl: './registration.component.html'
})
export class RegistrationComponent {
  showPassword = false;
  password = '';
  strengthLabel = '';
  strengthClass = '';

  togglePassword(): void {
    this.showPassword = !this.showPassword;
  }

  updateStrength(value: string): void {
    this.password = value;
    if (!value) {
      this.strengthLabel = '';
      this.strengthClass = '';
      return;
    }
    let score = 0;
    if (value.length >= 8) score += 1;
    if (/[A-Z]/.test(value)) score += 1;
    if (/[0-9]/.test(value)) score += 1;
    if (/[^A-Za-z0-9]/.test(value)) score += 1;

    const labels = ['Weak', 'Fair', 'Good', 'Strong'];
    const classes = ['weak', 'fair', 'good', 'strong'];
    const idx = Math.max(score - 1, 0);
    this.strengthLabel = labels[idx] ?? '';
    this.strengthClass = classes[idx] ?? '';
  }

  strengthActive(index: number): boolean {
    if (!this.password || !this.strengthClass) return false;
    const levels = ['weak', 'fair', 'good', 'strong'];
    const current = levels.indexOf(this.strengthClass);
    return current >= index;
  }
}
