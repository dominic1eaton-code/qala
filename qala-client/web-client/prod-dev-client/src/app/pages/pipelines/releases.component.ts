import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-releases',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './releases.component.html',
  styleUrl: './releases.component.css'
})
export class PipelinesReleasesComponent {}
