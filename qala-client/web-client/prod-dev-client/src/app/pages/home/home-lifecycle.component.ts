import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-home-lifecycle',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './home-lifecycle.component.html',
  styleUrl: './home.styles.css'
})
export class HomeLifecycleComponent {}
