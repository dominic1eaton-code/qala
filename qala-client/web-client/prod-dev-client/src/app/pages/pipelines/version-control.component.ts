import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-version-control',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './version-control.component.html',
  styleUrl: './version-control.component.css'
})
export class PipelinesVersionControlComponent {}
