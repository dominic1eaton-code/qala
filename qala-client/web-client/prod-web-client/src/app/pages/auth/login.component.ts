import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-login',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './login.component.html',
  styleUrl: './auth.styles.css'
})
export class LoginComponent {
  showPassword = false;

  togglePassword(): void {
    this.showPassword = !this.showPassword;
  }
}


