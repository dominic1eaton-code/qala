import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-environments',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './environments.component.html',
  styleUrl: './environments.component.css'
})
export class FactoryEnvironmentsComponent {}
