import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-release-schedule',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './release-schedule.component.html',
  styleUrl: './release-schedule.component.css'
})
export class PipelinesReleaseScheduleComponent {}
