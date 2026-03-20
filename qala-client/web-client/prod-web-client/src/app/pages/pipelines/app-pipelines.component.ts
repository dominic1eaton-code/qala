import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-app-pipelines',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './app-pipelines.component.html',
  styleUrl: './app-pipelines.component.css'
})
export class PipelinesAppComponent {}
