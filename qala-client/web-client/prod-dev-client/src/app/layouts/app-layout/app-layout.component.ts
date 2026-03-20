import { Component, inject } from '@angular/core';
import { NavigationEnd, Router, RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';
import { AsyncPipe, NgFor, NgIf } from '@angular/common';
import { filter, map, startWith } from 'rxjs/operators';
import { NAV_SYSTEMS } from './nav.config';

@Component({
  selector: 'app-layout',
  standalone: true,
  imports: [RouterOutlet, RouterLink, RouterLinkActive, NgFor, NgIf, AsyncPipe],
  templateUrl: './app-layout.component.html'
})
export class AppLayoutComponent {
  private readonly router = inject(Router);
  readonly navSystems = NAV_SYSTEMS;
  readonly activeSystem$ = this.router.events.pipe(
    filter((event): event is NavigationEnd => event instanceof NavigationEnd),
    map(() => this.stripUrl(this.router.url)),
    startWith(this.stripUrl(this.router.url)),
    map((url) => this.navSystems.find((system) => url.startsWith(system.route)))
  );

  private stripUrl(url: string): string {
    return url.split('?')[0].split('#')[0];
  }
}
