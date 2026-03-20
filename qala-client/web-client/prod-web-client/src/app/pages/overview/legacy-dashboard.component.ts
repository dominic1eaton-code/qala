import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-legacy-dashboard',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './legacy-dashboard.component.html',
  styleUrl: './legacy-dashboard.component.css'
})
export class OverviewLegacyDashboardComponent {}
