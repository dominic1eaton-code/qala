import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-home-factory',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './home-factory.component.html',
  styleUrl: './home.styles.css'
})
export class HomeFactoryComponent {}
