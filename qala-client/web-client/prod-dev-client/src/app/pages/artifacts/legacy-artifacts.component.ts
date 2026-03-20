import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-legacy-artifacts',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './legacy-artifacts.component.html',
  styleUrl: './legacy-artifacts.component.css'
})
export class ArtifactsLegacyComponent {}
