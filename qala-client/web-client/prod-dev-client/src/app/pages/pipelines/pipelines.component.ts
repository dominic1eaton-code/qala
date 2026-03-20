import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-pipelines',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './pipelines.component.html',
  styleUrl: './pipelines.component.css'
})
export class PipelinesOverviewComponent {}
