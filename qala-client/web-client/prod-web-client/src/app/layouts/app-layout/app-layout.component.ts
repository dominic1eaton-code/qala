import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';
import { NgFor } from '@angular/common';
import { NAV_SYSTEMS } from './nav.config';

@Component({
  selector: 'app-layout',
  standalone: true,
  imports: [RouterOutlet, RouterLink, RouterLinkActive, NgFor],
  templateUrl: './app-layout.component.html'
})
export class AppLayoutComponent {
  readonly navSystems = NAV_SYSTEMS;
}
