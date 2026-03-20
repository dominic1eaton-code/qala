import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-app-dashboard',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './app-dashboard.component.html',
  styleUrl: './app-dashboard.component.css'
})
export class OverviewAppDashboardComponent {}
