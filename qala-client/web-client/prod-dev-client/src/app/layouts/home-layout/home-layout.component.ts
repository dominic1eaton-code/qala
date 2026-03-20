import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-home-layout',
  standalone: true,
  imports: [RouterOutlet, RouterLink, RouterLinkActive],
  templateUrl: './home-layout.component.html',
  styleUrl: '../../pages/home/home.styles.css'
})
export class HomeLayoutComponent {}
