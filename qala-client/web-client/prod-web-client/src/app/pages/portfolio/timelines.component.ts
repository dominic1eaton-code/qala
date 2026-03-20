import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-timelines',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './timelines.component.html',
  styleUrl: './timelines.component.css'
})
export class PortfolioTimelinesComponent {}
