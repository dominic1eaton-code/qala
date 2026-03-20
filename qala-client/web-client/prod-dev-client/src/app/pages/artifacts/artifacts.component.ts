import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-artifacts',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './artifacts.component.html',
  styleUrl: './artifacts.component.css'
})
export class ArtifactsOverviewComponent {}
